---
id: doc-6kb
title: 'AGUI: Deployment & Packaging Strategy'
type: guide
scope: internal
created_at: '2026-01-09T00:53:25.840105Z'
updated_at: '2026-01-09T00:53:25.840105Z'
---

# AGUI: Deployment & Packaging Strategy

## Overview

AGUI is distributed as native desktop applications across multiple platforms using `cargo-dist`. This guide documents the packaging strategy, platform-specific artifacts, and deployment process.

## Target Platforms

- **Linux**: AppImage, DEB, RPM
- **macOS**: DMG
- **Windows**: MSI

## Packaging Tool: cargo-dist

We use `cargo-dist` to build and distribute multi-platform binaries. This ensures consistent, reproducible builds across all platforms.

### Key Features

- Automated multi-platform binary building
- Code signing and notarization support
- Update checking and distribution
- Smoke testing of generated artifacts

## Build Artifacts

Each release generates:
1. Platform-specific installers (AppImage, DMG, MSI)
2. Checksums and signatures
3. Release notes

## Smoke Testing

All generated artifacts must pass smoke tests:
- Application launches without errors
- Basic UI renders correctly
- Network connectivity can be established
- Shutdown is graceful

## Distribution Channels

- Direct download from releases page
- Auto-update mechanism via cargo-dist update checks
- Platform-specific package managers (apt, brew, winget)

## Maintenance & Patching

Updates follow semantic versioning. Critical security patches are distributed immediately; feature releases follow a regular schedule.

## Related Documentation

- AGUI Rust Client - Implementation Plan (Phase 1 scaffolding)
- Universal Agent GUI - Rust Desktop Specification v1.1
