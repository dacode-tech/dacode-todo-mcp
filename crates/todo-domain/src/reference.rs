//! External references (spec §3, D10): `{kind, target, note?}` entries
//! on epics, tasks, decisions and docs.
//!
//! A [`References`] collection keeps the canonical sort §8 demands —
//! `(kind, target, note)` — under out-of-order insertion: it is a
//! [`BTreeSet`], so insertion order never reaches the serialized bytes,
//! and exact duplicates (all three fields equal) deduplicate on insert.
//! Kind order is the enum's declaration order, which spells §3's
//! enumeration `tracker, url, doc, spec, file, other`.
//!
//! Validation is deliberately loose (a judgment call; §3 pins only the
//! shape): the target must be a non-empty, non-whitespace string, and
//! `#anchor` suffixes are accepted for `doc`-kind targets only
//! (`camera:S1#5.2`) — exactly one `#`, with a non-empty base and
//! anchor. Targets are otherwise opaque: a `url` target is not
//! URL-parsed and a `doc` base is not parsed as a [`crate::DocId`],
//! because references may cite docs of other repos or not-yet-registered
//! addresses.
//!
//! A [`Reference`] is valid by construction: [`Reference::new`] and
//! deserialization (via `#[serde(try_from)]`) run the same checks, so an
//! invalid reference is not representable and a load rejects exactly
//! what the constructor rejects. [`References::validate`] re-checks a
//! whole collection as a cheap invariant guard.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ValueError;

/// The kind of a reference target (spec §3, D10). Declaration order is
/// the total order used by the canonical `(kind, target, note)` sort;
/// serde spelling is `snake_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    /// An external issue tracker entry (e.g. a JIRA key).
    Tracker,
    /// A URL.
    Url,
    /// A registered document ([`crate::DocId`]), optionally with a
    /// `#anchor` section suffix.
    Doc,
    /// A spec section citation.
    Spec,
    /// A file path.
    File,
    /// Anything else.
    Other,
}

impl ReferenceKind {
    /// The stable wire spelling of this kind, e.g. `"tracker"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tracker => "tracker",
            Self::Url => "url",
            Self::Doc => "doc",
            Self::Spec => "spec",
            Self::File => "file",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for ReferenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One `{kind, target, note?}` reference entry. The fields are private
/// and both construction paths — [`Reference::new`] and `Deserialize` —
/// run the same checks, so an invalid reference is not representable
/// (see the module docs).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "ReferenceWire")]
pub struct Reference {
    kind: ReferenceKind,
    target: String,
    note: Option<String>,
}

/// Deserialization shadow of [`Reference`]: the same wire shape, but
/// unvalidated. `TryFrom` runs [`Reference::new`]'s checks, so loading
/// rejects exactly what the constructor rejects — the format-agnostic
/// equivalent of the string newtypes' validating `Deserialize`.
#[derive(Deserialize)]
struct ReferenceWire {
    kind: ReferenceKind,
    target: String,
    note: Option<String>,
}

impl TryFrom<ReferenceWire> for Reference {
    type Error = ValueError;

    fn try_from(wire: ReferenceWire) -> Result<Self, Self::Error> {
        Self::new(wire.kind, wire.target, wire.note)
    }
}

impl Reference {
    /// Validates and constructs a reference. `note` must be `None` or a
    /// non-empty, non-whitespace string — an absent note is `None`, not
    /// an empty one.
    pub fn new(
        kind: ReferenceKind,
        target: String,
        note: Option<String>,
    ) -> Result<Self, ValueError> {
        check(kind, &target, note.as_deref())?;
        Ok(Self { kind, target, note })
    }

    /// The reference kind.
    pub fn kind(&self) -> ReferenceKind {
        self.kind
    }

    /// The target string, including any `#anchor` suffix.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// The optional human note.
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// Re-checks the construction rules — the shared checker behind
    /// [`Reference::new`], deserialization, and [`References::validate`].
    pub fn validate(&self) -> Result<(), ValueError> {
        check(self.kind, &self.target, self.note.as_deref())
    }
}

/// The rules of [`Reference::new`], shared with the re-validation path.
fn check(kind: ReferenceKind, target: &str, note: Option<&str>) -> Result<(), ValueError> {
    let invalid = |reason: &str| ValueError::InvalidReference {
        reason: reason.to_owned(),
        target: target.to_owned(),
    };
    if target.is_empty() || target.trim().is_empty() {
        return Err(invalid("target must not be empty or whitespace"));
    }
    if note.is_some_and(|n| n.trim().is_empty()) {
        return Err(invalid("note must not be empty or whitespace; use None"));
    }
    match (kind, target.split_once('#')) {
        // Anchors are a doc-kind notation (spec §3: `camera:S1#5.2`).
        (_, Some(_)) if kind != ReferenceKind::Doc => {
            Err(invalid("anchors are only valid on doc-kind references"))
        }
        // split_once cuts at the first '#', so extra '#' land in the
        // anchor: exactly one separator means the anchor holds none.
        (ReferenceKind::Doc, Some((base, anchor))) => {
            if anchor.contains('#') || base.trim().is_empty() || anchor.trim().is_empty() {
                Err(invalid("doc anchor must be a single non-empty #suffix"))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

/// A canonically sorted set of references: `(kind, target, note)`
/// ascending, deduplicated, stable under out-of-order insertion (§8).
/// Serializes as a sequence in that order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct References(std::collections::BTreeSet<Reference>);

impl References {
    /// An empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a reference, keeping the canonical order. Returns
    /// `false` without changing the collection if an equal entry was
    /// already present (deduplication is part of the contract).
    /// Re-checks the invariant, so the set only ever holds valid
    /// references.
    pub fn insert(&mut self, reference: Reference) -> Result<bool, ValueError> {
        reference.validate()?;
        Ok(self.0.insert(reference))
    }

    /// Iterates in canonical `(kind, target, note)` order.
    pub fn iter(&self) -> std::collections::btree_set::Iter<'_, Reference> {
        self.0.iter()
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the collection holds no entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Re-checks every entry's construction rules, in canonical order —
    /// a cheap invariant guard over the whole collection.
    pub fn validate(&self) -> Result<(), ValueError> {
        for reference in &self.0 {
            reference.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(kind: ReferenceKind, target: &str, note: Option<&str>) -> Reference {
        Reference::new(kind, target.to_owned(), note.map(str::to_owned)).unwrap()
    }

    #[test]
    fn canonical_order_survives_out_of_order_insertion() {
        let mut refs = References::new();
        // Inserted in scrambled kind/target/note order.
        for r in [
            reference(ReferenceKind::Url, "https://b.example", None),
            reference(ReferenceKind::Tracker, "CAM-2", Some("second note")),
            reference(ReferenceKind::Doc, "camera:S1#5.2", None),
            reference(ReferenceKind::Tracker, "CAM-1", None),
            reference(ReferenceKind::Url, "https://a.example", None),
            reference(ReferenceKind::Tracker, "CAM-1", Some("a note")),
        ] {
            assert!(refs.insert(r).unwrap());
        }
        let order: Vec<(ReferenceKind, &str, Option<&str>)> = refs
            .iter()
            .map(|r| (r.kind(), r.target(), r.note()))
            .collect();
        assert_eq!(
            order,
            vec![
                // §3 enumeration order: tracker < url < doc < spec < file < other
                (ReferenceKind::Tracker, "CAM-1", None),
                (ReferenceKind::Tracker, "CAM-1", Some("a note")),
                (ReferenceKind::Tracker, "CAM-2", Some("second note")),
                (ReferenceKind::Url, "https://a.example", None),
                (ReferenceKind::Url, "https://b.example", None),
                (ReferenceKind::Doc, "camera:S1#5.2", None),
            ]
        );
    }

    #[test]
    fn exact_duplicates_deduplicate_on_insert() {
        let mut refs = References::new();
        let r = || reference(ReferenceKind::Tracker, "CAM-1", Some("note"));
        assert!(refs.insert(r()).unwrap());
        assert!(!refs.insert(r()).unwrap(), "duplicate reports not-inserted");
        assert_eq!(refs.len(), 1);
        // A different note on the same target is a distinct entry.
        assert!(
            refs.insert(reference(ReferenceKind::Tracker, "CAM-1", Some("other")))
                .unwrap()
        );
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn doc_anchors_accepted_only_for_doc_kind() {
        assert_eq!(
            reference(ReferenceKind::Doc, "camera:S1#5.2", None).target(),
            "camera:S1#5.2"
        );
        // Plain doc targets are fine.
        reference(ReferenceKind::Doc, "camera:S1", None);
        for (kind, target) in [
            (ReferenceKind::Url, "https://x.example#frag"),
            (ReferenceKind::Tracker, "CAM-1#2"),
            (ReferenceKind::Spec, "§3#anchor"),
            (ReferenceKind::File, "src/main.rs#L10"),
            (ReferenceKind::Other, "x#y"),
        ] {
            assert_eq!(
                Reference::new(kind, target.to_owned(), None),
                Err(ValueError::InvalidReference {
                    reason: "anchors are only valid on doc-kind references".to_owned(),
                    target: target.to_owned(),
                }),
                "{kind} target {target:?} must be rejected"
            );
        }
    }

    #[test]
    fn malformed_doc_anchors_rejected() {
        for target in [
            "camera:S1#",    // empty anchor
            "#5.2",          // empty base
            "camera:S1#a#b", // more than one anchor
            "camera:S1#   ", // whitespace anchor
        ] {
            assert!(
                matches!(
                    Reference::new(ReferenceKind::Doc, target.to_owned(), None),
                    Err(ValueError::InvalidReference { .. })
                ),
                "doc target {target:?} must be rejected"
            );
        }
    }

    #[test]
    fn empty_targets_and_notes_rejected() {
        for target in ["", " ", "\t"] {
            assert!(matches!(
                Reference::new(ReferenceKind::Other, target.to_owned(), None),
                Err(ValueError::InvalidReference { .. })
            ));
        }
        for note in [Some(""), Some("  ")] {
            assert!(matches!(
                Reference::new(
                    ReferenceKind::Other,
                    "x".to_owned(),
                    note.map(str::to_owned)
                ),
                Err(ValueError::InvalidReference { .. })
            ));
        }
    }

    #[test]
    fn kind_order_follows_declaration_order() {
        assert!(ReferenceKind::Tracker < ReferenceKind::Url);
        assert!(ReferenceKind::Url < ReferenceKind::Doc);
        assert!(ReferenceKind::Doc < ReferenceKind::Spec);
        assert!(ReferenceKind::Spec < ReferenceKind::File);
        assert!(ReferenceKind::File < ReferenceKind::Other);
        assert_eq!(ReferenceKind::Tracker.as_str(), "tracker");
        assert_eq!(ReferenceKind::Other.to_string(), "other");
    }

    #[test]
    fn getters_round_out_the_private_fields() {
        let r = reference(
            ReferenceKind::Spec,
            "todo-mcp-design.md §3",
            Some("the model"),
        );
        assert_eq!(r.kind(), ReferenceKind::Spec);
        assert_eq!(r.target(), "todo-mcp-design.md §3");
        assert_eq!(r.note(), Some("the model"));
        assert!(r.validate().is_ok());
    }

    #[test]
    fn deserialize_rejects_what_new_rejects() {
        use crate::testcodec::{from_bytes, to_bytes};
        use serde::Serialize;

        // Same field order as the private ReferenceWire shadow; the test
        // codec is positional, so this serializes to exactly the bytes a
        // Reference of the same values would.
        #[derive(Serialize)]
        struct WireRef {
            kind: ReferenceKind,
            target: String,
            note: Option<String>,
        }

        let bad = [
            // empty target
            WireRef {
                kind: ReferenceKind::Url,
                target: String::new(),
                note: None,
            },
            // whitespace-only target
            WireRef {
                kind: ReferenceKind::Url,
                target: "   ".to_owned(),
                note: None,
            },
            // anchor on a non-doc kind
            WireRef {
                kind: ReferenceKind::Url,
                target: "https://x.example#frag".to_owned(),
                note: None,
            },
            // doc target with two anchors
            WireRef {
                kind: ReferenceKind::Doc,
                target: "camera:S1#a#b".to_owned(),
                note: None,
            },
            // doc target with an empty anchor
            WireRef {
                kind: ReferenceKind::Doc,
                target: "camera:S1#".to_owned(),
                note: None,
            },
            // empty note (must be None)
            WireRef {
                kind: ReferenceKind::Tracker,
                target: "CAM-1".to_owned(),
                note: Some(String::new()),
            },
        ];
        for w in &bad {
            let one = to_bytes(w).unwrap();
            assert!(
                from_bytes::<Reference>(&one).is_err(),
                "Reference must reject target {:?}",
                w.target
            );
            // The same bad element inside a References set is rejected too.
            let set = to_bytes(&vec![w]).unwrap();
            assert!(
                from_bytes::<References>(&set).is_err(),
                "References must reject target {:?}",
                w.target
            );
        }

        // A good reference still round-trips through Deserialize.
        let good = reference(ReferenceKind::Doc, "camera:S1#5.2", Some("the model"));
        assert_eq!(
            from_bytes::<Reference>(&to_bytes(&good).unwrap()).unwrap(),
            good
        );
    }
}
