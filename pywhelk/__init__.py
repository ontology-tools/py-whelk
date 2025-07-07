__version__ = "0.1.0"



def create_reasoner(ontology):
    """
    Create a reasoner instance.
    """
    from pyhornedowl import create_reasoner
    import platform
    from os import path, listdir
    
    dir = path.join(path.dirname(__file__), "pywhelk")
    libname = [f for f in listdir(dir) if any(f.endswith(s) for s in [".so", ".dylib", ".dll"])][0]
    libpath = path.abspath(path.join(dir, libname))
    

    return create_reasoner(libpath, ontology)