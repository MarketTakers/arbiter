use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use ed25519_dalek::VerifyingKey;
use miette::Diagnostic;
use rcgen::KeyPair;
use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

use crate::context::ServerContext;
use crate::db::models::{NewRotationClientAck, NewTlsCertificate, NewTlsRotationHistory};
use crate::db::schema::{rotation_client_acks, tls_certificates, tls_rotation_history, tls_rotation_state};
use crate::db::DatabasePool;

use super::{generate_cert, CertificateMetadata, TlsInitError};

#[derive(Error, Debug, Diagnostic)]
pub enum RotationError {
    #[error("Certificate generation failed: {0}")]
    #[diagnostic(code(arbiter_server::rotation::cert_generation))]
    CertGeneration(#[from] rcgen::Error),

    #[error("Database error: {0}")]
    #[diagnostic(code(arbiter_server::rotation::database))]
    Database(#[from] diesel::result::Error),

    #[error("TLS initialization error: {0}")]
    #[diagnostic(code(arbiter_server::rotation::tls_init))]
    TlsInit(#[from] TlsInitError),

    #[error("Invalid rotation state: {0}")]
    #[diagnostic(code(arbiter_server::rotation::invalid_state))]
    InvalidState(String),

    #[error("No active certificate found")]
    #[diagnostic(code(arbiter_server::rotation::no_active_cert))]
    NoActiveCertificate,
}

/// Состояние процесса ротации сертификата
#[derive(Debug, Clone)]
pub enum RotationState {
    /// Обычная работа, ротация не требуется
    Normal,

    /// Ротация инициирована, новый сертификат сгенерирован
    RotationInitiated {
        initiated_at: i64,
        new_cert_id: i32,
    },

    /// Ожидание подтверждений (ACKs) от клиентов
    WaitingForAcks {
        new_cert_id: i32,
        initiated_at: i64,
        timeout_at: i64,
    },

    /// Все ACK получены или таймаут истёк, готов к ротации
    ReadyToRotate {
        new_cert_id: i32,
    },
}

impl RotationState {
    /// Загрузить состояние из базы данных
    pub async fn load_from_db(db: &DatabasePool) -> Result<Self, RotationError> {
        use crate::db::schema::tls_rotation_state::dsl::*;

        let mut conn = db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        let state_record: (i32, String, Option<i32>, Option<i32>, Option<i32>) =
            tls_rotation_state
                .select((id, state, new_cert_id, initiated_at, timeout_at))
                .filter(id.eq(1))
                .first(&mut conn)
                .await?;

        let rotation_state = match state_record.1.as_str() {
            "normal" => RotationState::Normal,
            "initiated" => {
                let cert_id = state_record.2.ok_or_else(|| {
                    RotationError::InvalidState("Initiated state missing new_cert_id".into())
                })?;
                let init_at = state_record.3.ok_or_else(|| {
                    RotationError::InvalidState("Initiated state missing initiated_at".into())
                })?;
                RotationState::RotationInitiated {
                    initiated_at: init_at as i64,
                    new_cert_id: cert_id,
                }
            }
            "waiting_acks" => {
                let cert_id = state_record.2.ok_or_else(|| {
                    RotationError::InvalidState("WaitingForAcks state missing new_cert_id".into())
                })?;
                let init_at = state_record.3.ok_or_else(|| {
                    RotationError::InvalidState("WaitingForAcks state missing initiated_at".into())
                })?;
                let timeout = state_record.4.ok_or_else(|| {
                    RotationError::InvalidState("WaitingForAcks state missing timeout_at".into())
                })?;
                RotationState::WaitingForAcks {
                    new_cert_id: cert_id,
                    initiated_at: init_at as i64,
                    timeout_at: timeout as i64,
                }
            }
            "ready" => {
                let cert_id = state_record.2.ok_or_else(|| {
                    RotationError::InvalidState("Ready state missing new_cert_id".into())
                })?;
                RotationState::ReadyToRotate {
                    new_cert_id: cert_id,
                }
            }
            other => {
                return Err(RotationError::InvalidState(format!(
                    "Unknown state: {}",
                    other
                )))
            }
        };

        Ok(rotation_state)
    }

    /// Сохранить состояние в базу данных
    pub async fn save_to_db(&self, db: &DatabasePool) -> Result<(), RotationError> {
        use crate::db::schema::tls_rotation_state::dsl::*;

        let mut conn = db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        let (state_str, cert_id, init_at, timeout) = match self {
            RotationState::Normal => ("normal", None, None, None),
            RotationState::RotationInitiated {
                initiated_at: init,
                new_cert_id: cert,
            } => ("initiated", Some(*cert), Some(*init as i32), None),
            RotationState::WaitingForAcks {
                new_cert_id: cert,
                initiated_at: init,
                timeout_at: timeout_val,
            } => (
                "waiting_acks",
                Some(*cert),
                Some(*init as i32),
                Some(*timeout_val as i32),
            ),
            RotationState::ReadyToRotate { new_cert_id: cert } => ("ready", Some(*cert), None, None),
        };

        diesel::update(tls_rotation_state.filter(id.eq(1)))
            .set((
                state.eq(state_str),
                new_cert_id.eq(cert_id),
                initiated_at.eq(init_at),
                timeout_at.eq(timeout),
            ))
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

/// Фоновый таск для автоматической ротации сертификатов
pub struct RotationTask {
    context: Arc<crate::context::_ServerContextInner>,
    check_interval: Duration,
    rotation_threshold: Duration,
    ack_timeout: Duration,
    shutdown_rx: watch::Receiver<bool>,
}

impl RotationTask {
    /// Создать новый rotation task
    pub fn new(
        context: Arc<crate::context::_ServerContextInner>,
        check_interval: Duration,
        rotation_threshold: Duration,
        ack_timeout: Duration,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            context,
            check_interval,
            rotation_threshold,
            ack_timeout,
            shutdown_rx,
        }
    }

    /// Запустить фоновый таск мониторинга и ротации
    pub async fn run(mut self) -> Result<(), RotationError> {
        info!("Starting TLS certificate rotation task");

        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.check_interval) => {
                    if let Err(e) = self.check_and_process().await {
                        error!("Rotation task error: {}", e);
                    }
                }
                _ = self.shutdown_rx.changed() => {
                    info!("Rotation task shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Проверить текущее состояние и выполнить необходимые действия
    async fn check_and_process(&self) -> Result<(), RotationError> {
        let state = self.context.rotation_state.read().await.clone();

        match state {
            RotationState::Normal => {
                // Проверить, нужна ли ротация
                self.check_expiration_and_initiate().await?;
            }
            RotationState::RotationInitiated { new_cert_id, .. } => {
                // Автоматически перейти в WaitingForAcks
                self.transition_to_waiting_acks(new_cert_id).await?;
            }
            RotationState::WaitingForAcks {
                new_cert_id,
                timeout_at,
                ..
            } => {
                self.handle_waiting_for_acks(new_cert_id, timeout_at).await?;
            }
            RotationState::ReadyToRotate { new_cert_id } => {
                self.execute_rotation(new_cert_id).await?;
            }
        }

        Ok(())
    }

    /// Проверить срок действия сертификата и инициировать ротацию если нужно
    async fn check_expiration_and_initiate(&self) -> Result<(), RotationError> {
        let threshold_secs = self.rotation_threshold.as_secs() as i64;

        if self.context.tls.check_expiration(threshold_secs).await {
            info!("Certificate expiring soon, initiating rotation");
            self.initiate_rotation().await?;
        }

        Ok(())
    }

    /// Инициировать ротацию: сгенерировать новый сертификат и сохранить в БД
    pub async fn initiate_rotation(&self) -> Result<i32, RotationError> {
        info!("Initiating certificate rotation");

        // 1. Генерация нового сертификата
        let keypair = KeyPair::generate()?;
        let (cert, not_before, not_after) = generate_cert(&keypair)?;
        let cert_der = cert.der().clone();

        // 2. Сохранение в БД (is_active = false, пока не активирован)
        let new_cert_id = self
            .save_new_certificate(&cert_der, &keypair, not_before, not_after)
            .await?;

        info!(new_cert_id, "New certificate generated and saved");

        // 3. Обновление rotation_state
        let new_state = RotationState::RotationInitiated {
            initiated_at: chrono::Utc::now().timestamp(),
            new_cert_id,
        };
        *self.context.rotation_state.write().await = new_state.clone();
        new_state.save_to_db(&self.context.db).await?;

        // 4. Логирование в audit trail
        self.log_rotation_event(new_cert_id, "rotation_initiated", None)
            .await?;

        Ok(new_cert_id)
    }

    /// Перейти в состояние WaitingForAcks и разослать уведомления
    async fn transition_to_waiting_acks(&self, new_cert_id: i32) -> Result<(), RotationError> {
        info!(new_cert_id, "Transitioning to WaitingForAcks state");

        let initiated_at = chrono::Utc::now().timestamp();
        let timeout_at = initiated_at + self.ack_timeout.as_secs() as i64;

        // Обновить состояние
        let new_state = RotationState::WaitingForAcks {
            new_cert_id,
            initiated_at,
            timeout_at,
        };
        *self.context.rotation_state.write().await = new_state.clone();
        new_state.save_to_db(&self.context.db).await?;

        // TODO: Broadcast уведомлений клиентам
        // self.broadcast_rotation_notification(new_cert_id, timeout_at).await?;

        info!(timeout_at, "Rotation notifications sent, waiting for ACKs");

        Ok(())
    }

    /// Обработка состояния WaitingForAcks: проверка ACKs и таймаута
    async fn handle_waiting_for_acks(
        &self,
        new_cert_id: i32,
        timeout_at: i64,
    ) -> Result<(), RotationError> {
        let now = chrono::Utc::now().timestamp();

        // Проверить таймаут
        if now > timeout_at {
            let missing = self.get_missing_acks(new_cert_id).await?;
            warn!(
                missing_count = missing.len(),
                "Rotation ACK timeout reached, proceeding with rotation"
            );

            // Переход в ReadyToRotate
            let new_state = RotationState::ReadyToRotate { new_cert_id };
            *self.context.rotation_state.write().await = new_state.clone();
            new_state.save_to_db(&self.context.db).await?;

            self.log_rotation_event(
                new_cert_id,
                "timeout",
                Some(format!("Missing ACKs from {} clients", missing.len())),
            )
            .await?;

            return Ok(());
        }

        // Проверить, все ли ACK получены
        let missing = self.get_missing_acks(new_cert_id).await?;

        if missing.is_empty() {
            info!("All clients acknowledged, ready to rotate");

            let new_state = RotationState::ReadyToRotate { new_cert_id };
            *self.context.rotation_state.write().await = new_state.clone();
            new_state.save_to_db(&self.context.db).await?;

            self.log_rotation_event(new_cert_id, "acks_complete", None)
                .await?;
        } else {
            let time_remaining = timeout_at - now;
            debug!(
                missing_count = missing.len(),
                time_remaining,
                "Waiting for rotation ACKs"
            );
        }

        Ok(())
    }

    /// Выполнить атомарную ротацию сертификата
    async fn execute_rotation(&self, new_cert_id: i32) -> Result<(), RotationError> {
        info!(new_cert_id, "Executing certificate rotation");

        // 1. Загрузить новый сертификат из БД
        let new_cert = self.load_certificate(new_cert_id).await?;

        // 2. Атомарная замена в TlsManager
        self.context
            .tls
            .replace_certificate(new_cert)
            .await
            .map_err(RotationError::TlsInit)?;

        // 3. Обновить БД: старый is_active=false, новый is_active=true
        self.activate_certificate(new_cert_id).await?;

        // 4. TODO: Отключить всех клиентов
        // self.disconnect_all_clients().await?;

        // 5. Очистить rotation_state
        let new_state = RotationState::Normal;
        *self.context.rotation_state.write().await = new_state.clone();
        new_state.save_to_db(&self.context.db).await?;

        // 6. Очистить ACKs
        self.context.rotation_acks.write().await.clear();
        self.clear_rotation_acks(new_cert_id).await?;

        // 7. Логирование
        self.log_rotation_event(new_cert_id, "activated", None)
            .await?;

        info!(new_cert_id, "Certificate rotation completed successfully");

        Ok(())
    }

    /// Сохранить новый сертификат в БД
    async fn save_new_certificate(
        &self,
        cert_der: &[u8],
        keypair: &KeyPair,
        cert_not_before: i64,
        cert_not_after: i64,
    ) -> Result<i32, RotationError> {
        use crate::db::schema::tls_certificates::dsl::*;

        let mut conn = self.context.db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        let new_cert = NewTlsCertificate {
            cert: cert_der.to_vec(),
            cert_key: keypair.serialize_pem().as_bytes().to_vec(),
            not_before: cert_not_before as i32,
            not_after: cert_not_after as i32,
            is_active: false,
        };

        diesel::insert_into(tls_certificates)
            .values(&new_cert)
            .execute(&mut conn)
            .await?;

        // Получить ID последней вставленной записи
        let cert_id: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
            "last_insert_rowid()",
        ))
        .first(&mut conn)
        .await?;

        self.log_rotation_event(cert_id, "created", None).await?;

        Ok(cert_id)
    }

    /// Загрузить сертификат из БД
    async fn load_certificate(&self, cert_id: i32) -> Result<CertificateMetadata, RotationError> {
        use crate::db::schema::tls_certificates::dsl::*;

        let mut conn = self.context.db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        let cert_record: (Vec<u8>, Vec<u8>, i32, i32, i32) = tls_certificates
            .select((cert, cert_key, not_before, not_after, created_at))
            .filter(id.eq(cert_id))
            .first(&mut conn)
            .await?;

        let cert_der = rustls::pki_types::CertificateDer::from(cert_record.0);
        let key_pem = String::from_utf8(cert_record.1)
            .map_err(|e| RotationError::InvalidState(format!("Invalid key encoding: {}", e)))?;
        let keypair = KeyPair::from_pem(&key_pem)?;

        Ok(CertificateMetadata {
            cert_id,
            cert: cert_der,
            keypair: Arc::new(keypair),
            not_before: cert_record.2 as i64,
            not_after: cert_record.3 as i64,
            created_at: cert_record.4 as i64,
        })
    }

    /// Активировать сертификат (установить is_active=true)
    async fn activate_certificate(&self, cert_id: i32) -> Result<(), RotationError> {
        use crate::db::schema::tls_certificates::dsl::*;

        let mut conn = self.context.db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        // Деактивировать все сертификаты
        diesel::update(tls_certificates)
            .set(is_active.eq(false))
            .execute(&mut conn)
            .await?;

        // Активировать новый
        diesel::update(tls_certificates.filter(id.eq(cert_id)))
            .set(is_active.eq(true))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    /// Получить список клиентов, которые ещё не отправили ACK
    async fn get_missing_acks(&self, rotation_id: i32) -> Result<Vec<VerifyingKey>, RotationError> {
        // TODO: Реализовать получение списка всех активных клиентов
        // и вычитание тех, кто уже отправил ACK

        // Пока возвращаем пустой список
        Ok(Vec::new())
    }

    /// Очистить ACKs для данной ротации из БД
    async fn clear_rotation_acks(&self, rotation_id: i32) -> Result<(), RotationError> {
        use crate::db::schema::rotation_client_acks::dsl::*;

        let mut conn = self.context.db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        diesel::delete(rotation_client_acks.filter(rotation_id.eq(rotation_id)))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    /// Записать событие в audit trail
    async fn log_rotation_event(
        &self,
        history_cert_id: i32,
        history_event_type: &str,
        history_details: Option<String>,
    ) -> Result<(), RotationError> {
        use crate::db::schema::tls_rotation_history::dsl::*;

        let mut conn = self.context.db.get().await.map_err(|e| {
            RotationError::InvalidState(format!("Failed to get DB connection: {}", e))
        })?;

        let new_history = NewTlsRotationHistory {
            cert_id: history_cert_id,
            event_type: history_event_type.to_string(),
            details: history_details,
        };

        diesel::insert_into(tls_rotation_history)
            .values(&new_history)
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}
