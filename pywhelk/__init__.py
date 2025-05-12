__version__ = "0.1.0"

def create_reasoner():
    """
    Create a reasoner instance.
    """
    from pyhornedowl import load_reasoner
    import platform
    from os import path

    filename = "libpywhelk.so" if platform.system() != "Windows" else "pywhelk.dll"

    return load_reasoner(path.join(path.dirname(__file__), filename))