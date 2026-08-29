# janim-backend

## Introduction

This package provides the Rust backend for [JAnim](https://github.com/jkjkil4/JAnim) and is mainly used to accelerate performance-critical parts of the project.

## Structure

- `src/relation/` provides:
    - item relationship management and maintains the constraint between `parents` and `children`
    - bit-based ancestors/descendants tracking and `computed` flags

- `src/compute/` provides some accelerated computations implemented in Rust

- `src/math/` provides `Quaternion` class