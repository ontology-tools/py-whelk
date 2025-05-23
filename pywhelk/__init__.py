__version__ = "0.1.0"

def create_reasoner(ontology):
    """
    Create a reasoner instance.
    """
    from pyhornedowl import create_reasoner
    import platform
    from os import path

    filename = "libpywhelk.so" if platform.system() != "Windows" else "pywhelk.dll"

    return create_reasoner(path.join(path.dirname(__file__), filename), ontology)