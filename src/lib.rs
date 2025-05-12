use horned_owl::model as ho;
use horned_owl::model::{ArcStr, Component, MutableOntology};
use std::collections::HashSet;

use horned_owl::ontology::set::SetOntology;
use whelk::whelk::reasoner;

use pyhornedowlreasoner::{Reasoner, ReasonerError};
use whelk::whelk::owl::translate_ontology;

pub struct PyWhelk();

#[unsafe(no_mangle)]
pub fn create_reasoner() -> Box<dyn Reasoner> {
    Box::new(PyWhelk::create_reasoner())
}


impl PyWhelk {
    fn create_reasoner() -> Self {
        PyWhelk()
    }
}

impl Reasoner for PyWhelk {
    fn classify(
        &self,
        ontology: &SetOntology<ArcStr>,
    ) -> Result<SetOntology<ArcStr>, ReasonerError> {
        let mut ontology: SetOntology<ArcStr> = ontology.clone();
        let inferred_components = self.infer(&ontology)?;

        for component in inferred_components {
            ontology.insert(ho::AnnotatedComponent {
                component,
                ann: Default::default(), // TODO: Add annotation stating this axiom is inferred
            });
        }

        Ok(ontology)
    }

    fn infer(
        &self,
        ontology: &SetOntology<ArcStr>,
    ) -> Result<HashSet<Component<ArcStr>>, ReasonerError> {
        let set_ontology: SetOntology<ArcStr> = ontology.clone();
        let build = ho::Build::<ArcStr>::new();

        let translated = translate_ontology(&set_ontology);
        let state = reasoner::assert(&translated);

        let inferred_components = state
            .named_subsumptions()
            .iter()
            .map(|(sub, sup)| {
                let sub: ho::ClassExpression<ArcStr> = build.class(sub.id.clone()).into();
                let sup: ho::ClassExpression<ArcStr> = build.class(sup.id.clone()).into();
                ho::Component::SubClassOf(ho::SubClassOf { sub, sup })
            })
            .collect();

        Ok(inferred_components)
    }
}
