---
title: Mermaid Diagram Test
author: md2pdf
---

# Mermaid Diagram Tests

Testing various mermaid diagram types.

## Simple Flowchart

```mermaid
graph LR
    A[Start] --> B[Process]
    B --> C[End]
```

## Flowchart with Decisions

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E
```

## Top-Down Flow

```mermaid
flowchart TD
    Start --> Stop
```

## Sequence Diagram

```mermaid
sequenceDiagram
    Alice->>Bob: Hello Bob
    Bob-->>Alice: Hi Alice
```

This document tests mermaid rendering via oxdraw.
