use horned_owl::model::{
    AnnotatedComponent, ArcAnnotatedComponent, ArcStr, ClassExpression, Component, SubClassOf,
};
use horned_owl::ontology::indexed::OntologyIndex;
use horned_owl::ontology::set::SetOntology;
use horned_owl::{model as ho, vocab};
use std::collections::HashSet;

use pyhornedowlreasoner::{Reasoner, ReasonerError};
use whelk::whelk::model::Axiom;
use whelk::whelk::owl::{translate_axiom, translate_ontology};
use whelk::whelk::reasoner::{assert, assert_append, ReasonerState};

pub struct PyWhelkReasoner {
    state: ReasonerState,
}

#[unsafe(no_mangle)]
pub fn create_reasoner(
    ontology: SetOntology<ArcStr>,
) -> Box<dyn Reasoner<ArcStr, ArcAnnotatedComponent>> {
    Box::new(PyWhelkReasoner::create_reasoner(ontology))
}

impl PyWhelkReasoner {
    fn create_reasoner(ontology: SetOntology<ArcStr>) -> Self {
        let translated = translate_ontology(&ontology);

        PyWhelkReasoner {
            state: assert(&translated),
        }
    }
}

impl OntologyIndex<ArcStr, ArcAnnotatedComponent> for PyWhelkReasoner {
    fn index_insert(&mut self, cmp: ArcAnnotatedComponent) -> bool {
        let translated = translate_axiom(&cmp.component)
            .into_iter()
            .filter_map(|c| match c.as_ref() {
                Axiom::ConceptInclusion(ci) => Some(ci.clone()),
                _ => None,
            })
            .collect();
        self.state = assert_append(&translated, &self.state);

        false
    }

    fn index_remove(&mut self, _cmp: &AnnotatedComponent<ArcStr>) -> bool {
        false
    }
}

impl Reasoner<ArcStr, ArcAnnotatedComponent> for PyWhelkReasoner {
    fn get_name(&self) -> String {
        "PyWhelk".to_string()
    }

    fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn inferred_axioms(&self) -> HashSet<Component<ArcStr>> {
        let build = ho::Build::<ArcStr>::new();

        self.state
            .named_subsumptions()
            .iter()
            .map(|(sub, sup)| {
                let sub: ClassExpression<ArcStr> = build.class(sub.id.clone()).into();
                let sup: ClassExpression<ArcStr> = build.class(sup.id.clone()).into();
                Component::SubClassOf(SubClassOf { sub, sup })
            })
            .collect()
    }

    fn is_consistent(&self) -> Result<bool, ReasonerError> {
        let build = ho::Build::<ArcStr>::new();
        self.is_entailed(&Component::SubClassOf(SubClassOf {
            sub: build.class(vocab::OWL::Thing.as_ref()).into(),
            sup: build.class(vocab::OWL::Nothing.as_ref()).into(),
        }))
        .map(|r| !r)
    }

    fn is_entailed(&self, cmp: &Component<ArcStr>) -> Result<bool, ReasonerError> {
        match cmp {
            Component::SubClassOf(SubClassOf {
                sub: ClassExpression::Class(sub),
                sup: ClassExpression::Class(sup),
            }) => Ok(self
                .state
                .named_subsumptions()
                .iter()
                .find(|(b, p)| sub.0.to_string() == b.id && sup.0.to_string() == p.id)
                .is_some()),
            c => Err(ReasonerError::Other(format!(
                "Cannot check entailment for component {:?}",
                c
            ))
            .into()),
        }
    }
}
