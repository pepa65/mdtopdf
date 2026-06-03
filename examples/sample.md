---
title: Sample Document
subtitle: Testing md2pdf Rust Rewrite
author: John Doe
date: 2025-01-01
organization: ACME Corp
toc: true
cover: true
font: Linux Libertine
font-size: 11
header-left: ACME Corp
header-right: Confidential
footer-center: Sample Document
page-numbers: true
---

# Introduction

This is a sample markdown document to test the **md2pdf** converter. It includes various markdown features to verify the conversion works correctly.

## Text Formatting

You can use *italic text*, **bold text**, and ***bold italic text***. You can also use `inline code` for technical terms.

Here's some ~~strikethrough~~ text.

## Lists

### Unordered List

- First item
- Second item
  - Nested item 1
  - Nested item 2
- Third item

### Ordered List

1. First step
2. Second step
3. Third step

### Task List

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

## Code Blocks

Here's a Rust code example:

```rust
fn main() {
    println!("Hello, md2pdf!");

    let numbers = vec![1, 2, 3, 4, 5];
    for n in numbers {
        println!("Number: {}", n);
    }
}
```

And some Python:

```python
def greet(name):
    """Greet someone."""
    return f"Hello, {name}!"

print(greet("World"))
```

## Blockquotes

> This is a blockquote. It can span multiple lines and include various formatting like **bold** and *italic* text.
>
> It can also have multiple paragraphs.

## Links and Images

Here's a [link to Typst](https://typst.app) documentation.

![Sample Image](image.png)

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headings | Done | All levels supported |
| Lists | Done | Nested and ordered |
| Code | Done | With syntax highlighting |
| Math | Planned | LaTeX support |

## Math Equations

Inline math: $E = mc^2$

Display math:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

## Layout Directives

[page-break]

# New Section After Page Break

This section appears on a new page.

[columns-2]

This content will be in a two-column layout. You can use this for side-by-side comparisons or to fit more content on a page.

[/columns]

## Conclusion

This document demonstrates the core features of the md2pdf converter. The Rust rewrite using Typst provides faster conversion and a smaller binary compared to the original .NET implementation.
