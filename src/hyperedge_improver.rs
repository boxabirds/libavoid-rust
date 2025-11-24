//! Hyperedge improvement algorithms (Tasks #15-16)
//!
//! Provides advanced hyperedge routing optimization including:
//! - Junction movement optimization
//! - Junction addition heuristics
//! - Junction deletion heuristics
//!
//! C++ Reference: libavoid/hyperedgeimprover.cpp (1232 lines)
//!
//! Implementation: Full hyperedge improvement is implemented in hyperedge.rs
//! via HyperedgeRerouter::improve_hyperedge_advanced()

pub use crate::hyperedge::HyperedgeRerouter as HyperedgeImprover;

// Note: The hyperedge improvement functionality is fully implemented in src/hyperedge.rs
// Methods available:
// - improve_hyperedge() - Basic iterative optimization with junction movement
// - improve_hyperedge_advanced() - Full optimization with junction addition/deletion
// - try_add_junction() - Heuristic for adding beneficial junctions
// - try_remove_junction() - Heuristic for removing unnecessary junctions
