# Working with Attachments

Status: Reference Documentation  
Last updated: 2026-01-13

## Overview

Qipu stores attachments (images, PDFs, diagrams, etc.) in the `.qipu/attachments/` directory. Attachments are referenced from notes using standard markdown links, keeping notes readable in any markdown viewer.

## Directory Structure

```
.qipu/
├── attachments/          # All attachment files
│   ├── diagram-v1.png
│   ├── paper.pdf
│   └── screenshot.jpg
├── notes/               # Your notes
└── mocs/                # Maps of Content
```

## Referencing Attachments

Use relative markdown links to reference attachments from your notes:

```markdown
---
id: qp-a1b2
title: System Architecture Notes
type: permanent
---

## Overview
See the architecture diagram below:

![System architecture](../attachments/architecture-v2.png)

For more details, refer to the [design document](../attachments/design-v3.pdf).
```

### Best Practices

1. **Use relative paths**: Always reference attachments relative to the note location
   - From `notes/`: `../attachments/filename.ext`
   - From `mocs/`: `../attachments/filename.ext`

2. **Descriptive filenames**: Use clear, versioned names
   - Good: `architecture-v2.png`, `user-flow-diagram.svg`
   - Avoid: `image1.png`, `Screenshot 2026-01-13.png`

3. **Alt text for images**: Always include descriptive alt text
   ```markdown
   ![User authentication flow diagram](../attachments/auth-flow.png)
   ```

4. **Organize by convention**: Consider prefixing by topic or note type
   - `oauth-flow-diagram.png`
   - `database-schema-v3.svg`
   - `paper-cryptography-whitepaper.pdf`

## File Organization

### Flat Structure (Recommended)

Keep all attachments in a single directory with descriptive names:

```
attachments/
├── auth-oauth-flow.png
├── auth-pkce-sequence.svg
├── db-schema-v2.png
└── paper-tls-rfc.pdf
```

**Advantages**:
- Simple to manage
- Easy to reference
- No broken links from restructuring

### Per-Note Subdirectories (Advanced)

For large stores with many attachments, organize by note ID:

```
attachments/
├── qp-a1b2/
│   ├── diagram-v1.png
│   └── screenshot.jpg
└── qp-c3d4/
    └── flowchart.svg
```

Reference with: `../attachments/qp-a1b2/diagram-v1.png`

**Advantages**:
- Clear ownership
- Easier to clean up when notes are deleted

**Disadvantages**:
- More complex directory structure
- May break links if note IDs change

## Size Guidelines

**Keep attachments reasonable in size:**

- **Images**: Prefer compressed formats (PNG, WebP, JPEG)
  - Aim for < 1 MB per image
  - Use vector formats (SVG) for diagrams when possible

- **Documents**: Keep PDFs and documents under 10 MB
  - For larger files, consider storing externally and linking via URL

- **Videos/Large Media**: Store externally (cloud storage, etc.) and use URLs in the `sources` field

### Why Size Matters

- **Git performance**: Large binary files bloat repository size and slow down clones
- **Version control**: Git doesn't diff binary files efficiently
- **Collaboration**: Smaller repositories are easier to share and sync

## External References

For large media or external resources, use the `sources` field instead of attachments:

```yaml
---
id: qp-e5f6
title: Machine Learning Lecture Notes
type: literature
sources:
  - url: https://example.com/ml-lecture.mp4
    title: "Introduction to Neural Networks"
    accessed: 2026-01-13
---
```

## Git Integration

**Recommended .gitignore patterns:**

```gitignore
# If using stealth mode
.qipu/

# If committing notes but want to exclude large attachments
.qipu/attachments/*.mp4
.qipu/attachments/*.mov
.qipu/attachments/*.zip
```

**What to commit:**
- Small images and diagrams (< 1 MB)
- PDFs and documents that are part of research
- SVG diagrams and charts

**What NOT to commit:**
- Large video files (use external hosting)
- Temporary screenshots (clean up periodically)
- Downloaded installers or binaries
- Generated/derived files that can be recreated

## Export Considerations

When using `qipu export`, note that:

- Attachment links are preserved as-is in the exported markdown
- Attachments are NOT automatically copied (current limitation)
- For self-contained exports, manually copy needed attachments

**Future enhancement**: `qipu export --with-attachments` may copy referenced attachments to an output directory.

## Common Workflows

### Capturing research with screenshots

```bash
# 1. Take screenshot and save to attachments
cp ~/Desktop/screenshot.png .qipu/attachments/oauth-error-example.png

# 2. Create literature note referencing the screenshot
qipu create "OAuth error troubleshooting" --type literature --open

# 3. In the note, add the reference
![OAuth error message](../attachments/oauth-error-example.png)
```

### Diagramming architecture

```bash
# 1. Create diagram (using draw.io, Excalidraw, etc.)
# 2. Export as SVG to attachments
cp ~/Downloads/architecture.svg .qipu/attachments/system-architecture-v2.svg

# 3. Reference in permanent note
qipu create "System architecture overview" --type permanent --open
```

### Attaching PDFs

```bash
# 1. Save PDF to attachments
cp ~/Downloads/whitepaper.pdf .qipu/attachments/zero-knowledge-proofs-paper.pdf

# 2. Create literature note
qipu create "Zero-knowledge proofs research" --type literature --open

# 3. In the note
```markdown
---
id: qp-x7y8
title: Zero-knowledge proofs research
type: literature
sources:
  - url: https://eprint.iacr.org/2023/001.pdf
    title: "Efficient Zero-Knowledge Proofs"
    accessed: 2026-01-13
---

Local copy: [PDF](../attachments/zero-knowledge-proofs-paper.pdf)

## Key Insights
...
```
\`\`\`

## Cleaning Up

Periodically review and clean up unused attachments:

```bash
# List all attachments
ls -lh .qipu/attachments/

# Search for attachment references in notes
rg "filename.png" .qipu/notes/ .qipu/mocs/

# Remove unreferenced attachments manually
rm .qipu/attachments/old-screenshot.png
```

**Future enhancement**: `qipu doctor` may warn about unreferenced attachments.

## Anti-Patterns

**Don't:**
- Embed large binary files (> 10 MB) in git repositories
- Use absolute paths (breaks portability)
- Store generated/derived files that can be recreated
- Use attachments for code snippets (embed in notes instead)
- Name files generically (`image1.png`, `doc.pdf`)

**Do:**
- Use descriptive, versioned filenames
- Compress images before adding
- Use external hosting for large media
- Include context in alt text and surrounding notes
- Clean up unused attachments periodically

## Questions?

For more information:
- See `specs/storage-format.md` for technical details
- Check `docs/usage-patterns.md` for workflow examples
- Run `qipu doctor` to validate store health
