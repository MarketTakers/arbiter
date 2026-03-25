# Client Wallet Access Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated client details screen under `Clients` where operators can view a client and manage the set of accessible EVM wallets.

**Architecture:** Keep the existing `Clients` list as the entry point and add a focused details route/screen for one `SdkClientEntry`. Use Riverpod providers for the wallet inventory, client-scoped access draft, and save mutation. Because the current proto surface does not expose client-wallet-access RPCs, implement the UI and provider boundaries with an explicit unsupported save path instead of faking persistence.

**Tech Stack:** Flutter, AutoRoute, hooks_riverpod/riverpod, flutter_test

---

### Task 1: Add focused tests for client-details draft behavior

**Files:**
- Create: `test/screens/dashboard/clients/details/client_wallet_access_controller_test.dart`
- Create: `test/screens/dashboard/clients/details/client_details_screen_test.dart`

- [ ] **Step 1: Write the failing controller test**
- [ ] **Step 2: Run the controller test to verify it fails**
- [ ] **Step 3: Write the failing screen test**
- [ ] **Step 4: Run the screen test to verify it fails**

### Task 2: Add client-details state and data helpers

**Files:**
- Create: `lib/providers/sdk_clients/details.dart`
- Create: `lib/providers/sdk_clients/details.g.dart`
- Create: `lib/providers/sdk_clients/wallet_access.dart`
- Create: `lib/providers/sdk_clients/wallet_access.g.dart`

- [ ] **Step 1: Add provider types for selected client lookup**
- [ ] **Step 2: Add provider/notifier types for wallet-access draft state**
- [ ] **Step 3: Implement unsupported save mutation boundary**
- [ ] **Step 4: Run controller tests to make them pass**

### Task 3: Build the client-details UI with granular widgets

**Files:**
- Create: `lib/screens/dashboard/clients/details/client_details.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/client_details_header.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/client_summary_card.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/wallet_access_section.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/wallet_access_search_field.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/wallet_access_list.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/wallet_access_tile.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/wallet_access_save_bar.dart`
- Create: `lib/screens/dashboard/clients/details/widgets/client_details_state_panel.dart`

- [ ] **Step 1: Build the screen shell and summary widgets**
- [ ] **Step 2: Build the wallet-access list/search/save widgets**
- [ ] **Step 3: Keep widget files granular and avoid hardcoded sizes**
- [ ] **Step 4: Run the screen tests to make them pass**

### Task 4: Wire navigation from the clients list

**Files:**
- Modify: `lib/router.dart`
- Modify: `lib/router.gr.dart`
- Modify: `lib/screens/dashboard/clients/table.dart`

- [ ] **Step 1: Add the client-details route**
- [ ] **Step 2: Add a row affordance to open the client-details screen**
- [ ] **Step 3: Keep the existing list usable as an overview**
- [ ] **Step 4: Run targeted screen tests again**

### Task 5: Regenerate code and verify the feature

**Files:**
- Modify: generated files as required by build tools

- [ ] **Step 1: Run code generation**
- [ ] **Step 2: Run widget/provider tests**
- [ ] **Step 3: Run Flutter analysis on touched code**
- [ ] **Step 4: Review for requirement coverage and report the backend save limitation clearly**
