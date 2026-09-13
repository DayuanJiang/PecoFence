# Vendored windows-composition

Based on Microsoft's `windows-composition` 0.100.0 from the windows-rs project.
The PecoFence patch exposes raw-object accessors on the Composition wrapper so the
renderer can use interfaces not otherwise exposed by the wrapper.

Upstream licensing is MIT OR Apache-2.0. Both original license texts are retained
in `license-mit` and `license-apache-2.0`.

The workspace selects this directory through `[patch.crates-io]`. Keep the changes
small and remove the patch when an upstream version provides the required accessors.
