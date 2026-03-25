# Client Wallet Access Design

Date: 2026-03-25
Status: Proposed

## Goal

Add a client-centric UI that lets an operator choose which EVM wallets are visible to a given SDK client.

The mental model is:

> For this SDK client, choose which wallets it can see.

This UI should live under the existing `Clients` area, not under `Wallets`, because the permission is being edited from the client's perspective.

## Current Context

The current Flutter app has:

- A top-level dashboard with `Wallets`, `Clients`, and `About`
- A `Clients` screen that currently acts as a registry/list of `SdkClientEntry`
- A `Wallets` screen that lists managed EVM wallets
- An EVM grant creation flow that still manually asks for `Client ID`

Relevant observations from the current codebase:

- `SdkClientEntry` is already a richer admin-facing object than `WalletEntry`
- `WalletEntry` is currently minimal and not suited to owning the relationship UI
- The `Clients` screen already presents expandable client rows, which makes it the most natural entry point for a details view

## Chosen Approach

Use a dedicated client details screen.

From the `Clients` list, the operator opens one client and lands on a screen dedicated to that client. That screen includes a wallet access section that shows:

- Client identity and metadata
- Current wallet access selection
- A searchable/selectable list of available wallets
- Save feedback and error states

This is preferred over inline editing or a modal because it scales better when more capabilities are added later, such as:

- Search
- Bulk actions
- Explanatory copy
- Access summaries
- Future permission categories beyond wallet visibility

## User Experience

### Entry

The operator starts on the existing `Clients` screen.

Each client row gains a clear affordance to open details, for example:

- Tapping the row
- A trailing button such as `Manage access`

The existing list remains the overview surface. Editing does not happen inline.

### Client Details Screen

The screen is focused on a single client and should contain:

1. A lightweight header with back navigation
2. A client summary section
3. A wallet access section
4. Save/status feedback

The wallet access section is the core interaction:

- Show all available EVM wallets
- Show which wallets are currently accessible to this client
- Allow toggling access on/off
- Allow filtering/searching wallets when the list grows
- Show empty/loading/error states

### Save Model

Use an explicit save action rather than auto-save.

Reasons:

- Permission changes are administrative and should feel deliberate
- Multiple checkbox changes can be staged together
- It creates a clear place for pending, success, and failure states

The screen should track:

- Original selection from the server
- Current local selection in the form
- Whether there are unsaved changes

## Information Architecture

### Navigation

Add a nested route under the dashboard clients area for client details.

Conceptually:

- `Clients` remains the list screen
- `Client Details` becomes the edit/manage screen for one client

This keeps the current top-level tabs intact and avoids turning wallet access into a global dashboard concern.

### Screen Ownership

Wallet visibility is owned by the client details screen, not by the wallets screen.

The wallets screen can remain focused on wallet inventory and wallet creation.

## State Management

Use Riverpod.

State should be split by concern instead of managed in one large widget:

- Provider for the client list
- Provider for the wallet list
- Provider for the selected client details data
- Provider or notifier for wallet-access editing state
- Mutation/provider for saving wallet access changes

Recommended shape:

- One provider fetches the wallet inventory
- One provider fetches wallet access for a specific client
- One notifier owns the editable selection set for the client details form
- One mutation performs save and refreshes dependent providers

The editing provider should expose:

- Current selected wallet identifiers
- Original selected wallet identifiers
- `hasChanges`
- `isSaving`
- Validation or request error message when relevant

This keeps the UI declarative and prevents the screen widget from holding all state locally.

## Data Model Assumptions

The UI assumes there is or will be a backend/API surface equivalent to:

- List SDK clients
- List EVM wallets
- Read wallet access entries for one client
- Replace or update wallet access entries for one client

The screen should work with wallet identifiers that are stable from the backend perspective. If the backend only exposes positional IDs today, that should be normalized before binding the UI tightly to list index order.

This is important because the current grant creation screen derives `walletId` from list position, which is not a robust long-term UI contract.

## Layout and Styling Constraints

Implementation must follow these constraints:

- Use Riverpod for screen state and mutations
- Do not hardcode widths and heights
- Prefer layout driven by padding, constraints, flex, wrapping, and intrinsic content
- Keep widgets granular; a widget should not exceed roughly 50 lines
- Do not place all client-details widgets into a single file
- Create a dedicated widgets folder for the client details screen
- Reuse existing UI patterns and helper widgets where it is reasonable, but do not force reuse when it harms clarity

Recommended implementation structure:

- `lib/screens/dashboard/clients/details/`
- `lib/screens/dashboard/clients/details/client_details.dart`
- `lib/screens/dashboard/clients/details/widgets/...`

## Widget Decomposition

The client details feature should be composed from small widgets with single responsibilities.

Suggested widget split:

- `ClientDetailsScreen`
- `ClientDetailsScaffold`
- `ClientDetailsHeader`
- `ClientSummaryCard`
- `WalletAccessSection`
- `WalletAccessSearchField`
- `WalletAccessList`
- `WalletAccessListItem`
- `WalletAccessEmptyState`
- `WalletAccessErrorState`
- `WalletAccessSaveBar`

If useful, existing generic state panels or cards from the current screens can be adapted or extracted, but only where that reduces duplication without making the code harder to follow.

## Interaction Details

### Client Summary

Display the client's:

- Name
- ID
- Version
- Description
- Public key summary
- Registration date

This gives the operator confidence that they are editing the intended client.

### Wallet Access List

Each wallet item should show enough identity to make selection safe:

- Human-readable label if one exists in the backend later
- Otherwise the wallet address
- Optional secondary metadata if available later

Each item should have a clear selected/unselected control, most likely a checkbox.

### Unsaved Changes

When the current selection differs from the original selection:

- Show a save bar or action row
- Enable `Save`
- Optionally show `Reset` or `Discard`

When there are no changes:

- Save action is disabled or visually deemphasized

### Loading and Errors

The screen should independently handle:

- Client not found
- Wallet list unavailable
- Wallet access unavailable
- Save failure
- Empty wallet inventory

These states should be explicit in the UI rather than collapsed into a blank screen.

## Reuse Guidance

Reasonable reuse candidates from the current codebase:

- Existing color/theme primitives
- Existing state/empty panels if they can be extracted cleanly
- Existing wallet formatting helpers, if they are generalized

Reuse should not be prioritized over good boundaries. If the existing widget is too coupled to another screen, create a new focused widget instead.

## Testing Strategy

Plan for widget and provider-level coverage.

At minimum, implementation should be testable for:

- Rendering client summary
- Rendering preselected wallet access
- Toggling wallet selection
- Dirty state detection
- Save success refresh flow
- Save failure preserving local edits
- Empty/loading/error states

Given the current test directory is empty, this feature is a good place to establish basic screen/provider tests rather than relying only on manual verification.

## Out of Scope

The following are not required for the first version unless backend requirements force them:

- Cross-client bulk editing
- Wallet-side permission management
- Audit history UI
- Role templates
- Non-EVM asset permissions

## Recommendation Summary

Implement wallet access management as a dedicated client details screen under `Clients`.

This gives the cleanest product model:

- `Clients` answers "who is this app/client?"
- `Wallet access` answers "what wallets can it see?"

It also gives the best technical path for Riverpod-managed state, granular widget decomposition, and future expansion without crowding the existing client list UI.
