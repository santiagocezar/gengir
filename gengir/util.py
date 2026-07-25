from lxml.etree import QName
import re
import keyword


def avoid_keyword(name: str):
    if keyword.iskeyword(name) or name == "print":
        return "_" + name
    return name


def get_doc(callable_tag):
    """Return docstring text for a callable"""
    for element in callable_tag:
        tag = QName(element)
        if tag.localname == "doc":
            return str(element.text).replace("\\x", "x")
    return ""


def make_safe(string):
    """Avoid having unicode characters in docstrings (such as uXXXX)"""
    return string.replace("\\u", "u").replace("\\U", "U")


def prettify(string):
    return re.sub(r"([\s]{3,80})", r"\n\1", string)


GIR_TO_NATIVE_TYPEMAP = {
    "gboolean": "bool",
    "gint": "int",
    "guint": "int",
    "gint8": "int",
    "guint8": "int",
    "gint16": "int",
    "guint16": "int",
    "gint32": "int",
    "guint32": "int",
    "gint64": "int",
    "guint64": "int",
    "gsize": "int",
    "gpointer": "typing.Any",
    "none": "None",
    "gchar": "str",
    "guchar": "str",
    "gchar*": "str",
    "guchar*": "str",
    "glong": "long",
    "gulong": "long",
    "glong64": "long",
    "gulong64": "long",
    "gshort": "int",
    "gushort": "int",
    "gshort64": "int",
    "gushort64": "int",
    "gfloat": "float",
    "gdouble": "float",
    "string": "str",
    "GString": "str",
    "utf8": "str",
}


def get_native_type(typename: str):
    """Convert a C type to a Python type"""
    typename = typename.replace("const ", "")
    return GIR_TO_NATIVE_TYPEMAP.get(typename, typename)


def get_type_from_node(element):
    for child in element:
        subtag = QName(child)
        if subtag.localname == "type":
            try:
                return get_native_type(child.attrib["name"]) or "typing.Any"
            except KeyError:
                continue

    return "typing.Any"


def get_imports_from_type(typ: str):
    return typ[: typ.index(".")] if "." in typ else None


def get_returntype(element):
    """Return the return-type of a callable"""
    for elem_property in element:
        tag = QName(elem_property)
        if tag.localname == "return-value":
            return_doc = get_doc(elem_property).replace("\n", " ").strip()
            return_type = get_type_from_node(elem_property)
            return (return_doc, return_type)
    return ("", "None")
