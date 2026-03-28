---
name: Widget decomposition and provider subscriptions
description: Prefer splitting screens into multiple focused files/widgets; each widget subscribes to its own relevant providers
type: feedback
---

Split screens into multiple smaller widgets across multiple files. Each widget should subscribe only to the providers it needs (`ref.watch` at lowest possible level), rather than having one large screen widget that watches everything and passes data down as parameters.

**Why:** Reduces unnecessary rebuilds; improves readability; each file has one clear responsibility.

**How to apply:** When building a new screen, identify which sub-widgets need their own provider subscriptions and extract them into separate files (e.g., `widgets/grant_card.dart` watches enrichment providers itself, rather than the screen doing it and passing resolved strings down).
