# Luminance Surfman

[![Crates.io](https://img.shields.io/crates/v/luminance-surfman.svg)](https://crates.io/crates/luminance-surfman)
[![Docs.rs](https://docs.rs/luminance-surfman/badge.svg)](https://docs.rs/luminance-surfman)
[![Katharos License](https://img.shields.io/badge/License-Katharos-blue)](https://github.com/katharostech/katharos-license)

A Surfman platform crate for the Luminance graphics API

This crate is useful in situtions where you do not have control over the window creation, and
you need to be able to create a Luminance surface after the window and event loop have already
been created.

This crate currently supports creating a Luminance surface from a winit window, but could also
be easily extended to allow you to create surfaces from a [raw window handle][rwh]. Open an
issue if you have that use case!

[rwh]: https://docs.rs/raw-window-handle/0.3.3/raw_window_handle/
