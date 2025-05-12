# py-whelk
A wrapper around [whelk-rs](https://github.com/INCATools/whelk-rs) to use the reasoner in Python using [py-horned-owl](https://github.com/ontology-tools/py-horned-owl/).

## Usage
```python
from pyhornedowl import open_ontology
import pywhelk

ontology = open_ontology("path/to/ontology.owl")
reasoner = pywhelk.create_reasoner()

# Use the reasoner to infer axioms
inferred_axioms = reasoner.infer(ontology)

# Use the reasoner to classify an ontology 
classified_ontology = reasoner.classify(ontology)

# Consistency checking is not supported at the moment
reasoner.consistency(ontology) # ValueError: NotImplemented
```

## Installation
Build the shared library with `cargo build --release` and copy the resulting .dll or .so files to the `py-whelk` directory. Then, install the Python package with `pip install .`:

```bash
make
pip install .
```