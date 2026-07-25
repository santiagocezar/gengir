import ast
from typing import Any, Literal, Optional, TypedDict, TYPE_CHECKING, Union

if TYPE_CHECKING:
    from lxml.etree import _Element

from gengir.util import (
    avoid_keyword,
    get_doc,
    get_returntype,
    make_safe,
    prettify,
    get_imports_from_type,
    get_type_from_node,
)

from lxml.etree import QName

XMLNS = "http://www.gtk.org/introspection/core/1.0"


class Param(TypedDict):
    name: str
    typ: str
    doc: Optional[str]
    variadic: Optional[bool]


class Func(TypedDict):
    name: str
    parameters: list[Param]
    returntype: str
    kind: Literal["static", "method", "staticmethod"]
    returndoc: Optional[str]
    doc: Optional[str]


class Var(TypedDict):
    name: str
    value: Any
    typ: Optional[str]
    doc: Optional[str]
    const: Optional[bool]


class Enum(TypedDict):
    name: str
    doc: str
    values: list[Var]


class Class(TypedDict):
    name: str
    doc: str
    bases: list[Union[str, "Class"]]
    fields: list[Var]
    methods: list[Func]


class ModuleAnalyzer:
    gi_include = set[str]()

    enums = list[En]

    def __init__(self, *, include_docs: bool = True) -> None:
        self.include_docs = include_docs

    def imports_from_type(self, typ: str):
        if typ := get_imports_from_type(typ):
            self.gi_include.add(typ)

    def get_parameter_doc(self, element):
        """Returns the doc of a parameter"""
        param_doc = ""
        if self.include_docs:
            for elem_property in element:
                tag = QName(elem_property)
                if tag.localname == "doc":
                    param_doc = (
                        element.text.replace("\\x", "x")
                        .encode("utf-8")
                        .replace("\n", " ")
                        .strip()
                    )
                    break

        return param_doc

    def get_parameters(self, element):
        """Return the parameters of a callable"""
        args = list[Param]()

        for elem_property in element:
            tag = QName(elem_property)
            if tag.localname == "parameters":
                for param in elem_property:
                    subtag = QName(param)
                    param_name: str

                    if subtag.localname == "instance-parameter":
                        param_name = "self"
                    else:
                        try:
                            param_name = param.attrib["name"]
                        except KeyError:
                            continue

                    param_name = avoid_keyword(param_name)
                    param_type = get_type_from_node(param)
                    param_doc = make_safe(get_doc(param).replace("\n", " ").strip())

                    is_variadic = param_name == "..."
                    if is_variadic:
                        param_name = "args"

                    if param_name not in [p["name"] for p in args]:
                        self.imports_from_type(param_type)

                        args.append(
                            Param(
                                name=param_name,
                                doc=param_doc,
                                typ=param_type,
                                variadic=is_variadic,
                            )
                        )

        return args

    def make_function_ast(
        self,
        name: str,
        parameters: list[Param],
        returntype: tuple[str, str],
        docstring: str,
        static: bool = False,
    ):
        """Returns a function as a AST node"""
        fun = ast.FunctionDef()
        fun.name = avoid_keyword(name)

        fun.decorator_list = []
        if static:
            fun.decorator_list.append(ast.Name("staticmethod"))

        fun.args = ast.arguments(
            posonlyargs=[],
            args=[],
            kwonlyargs=[],
            kw_defaults=[],
            defaults=[],
        )
        fun.returns = ast.alias(returntype[1])

        docstring += "\n\n"

        for param in parameters:
            if param["variadic"]:
                fun.args.vararg = ast.arg("args", ast.alias(param["typ"]))
            elif param["name"] == "self":
                fun.args.args.append(ast.arg("self"))
            else:
                fun.args.args.append(ast.arg(param["name"], param["typ"]))
                fun.args.defaults.append(ast.Constant(None))

            if self.include_docs:
                docstring += ":param {name}: {doc}\n".format(**param)

        if self.include_docs:
            docstring += ":return: {}".format(prettify(returntype[0]))

        self.imports_from_type(returntype[1])

        fun.body = [ast.Expr(ast.Constant(docstring)), ast.Pass()]

        return fun

    def extract_methods(self, class_tag):
        """Return methods from a class element"""

        methods = list[ast.FunctionDef]()

        for element in class_tag:
            tag = QName(element)
            if tag.localname in ("function", "method", "virtual-method", "constructor"):
                method_name: str = element.attrib["name"]
                if method_name == "print":
                    method_name += "_"

                docstring = get_doc(element)
                params = self.get_parameters(element)

                decoration = (
                    "" if any(p["name"] == "self" for p in params) else "staticmethod"
                )
                returntype = get_returntype(element)

                if tag.localname == "constructor" and method_name == "new":
                    params_init = list(params)
                    params_init.insert(0, Param(name="self", doc="", variadic=False))
                    methods.append(
                        self.make_function_ast(
                            "__init__",
                            [Param(name="self", doc="", variadic=False), *params],
                            ("", "None"),
                            docstring,
                        )
                    )

                methods.append(
                    self.make_function_ast(
                        method_name, params, returntype, docstring, decoration
                    )
                )
        return methods

    def extract_fields(self, record_tag):
        """Return fields from a record element"""
        fields_content = list[ast.AnnAssign]()

        for element in record_tag:
            tag = QName(element)
            if tag.localname == "field":
                field_name = element.attrib["name"]
                field_type = get_type_from_node(element)
                self.imports_from_type(field_type)

                fields_content.append(
                    ast.AnnAssign(
                        target=ast.Name(field_name),
                        annotation=ast.alias(field_type),
                        simple=1,
                    )
                )
        return fields_content

    def get_bases(self, klass: ast.ClassDef):
        parents = list[str]()
        for base in klass.bases:
            parents.append(ast.unparse(base))
        return parents

    def build_classes(self, classes: list[ast.ClassDef]):
        """Order classes with correct dependency order
        also return external imports
        """
        ordered_classes = list[ast.ClassDef]()

        local_parents = set[str]()
        written_classes = set[str]()
        all_classes = set[str]([klass.name for klass in classes])

        for klass in classes:
            parents = self.get_bases(klass)
            local_parents = local_parents.union(
                set(
                    [
                        class_parent
                        for class_parent in parents
                        if "." not in class_parent
                    ]
                )
            )

        while written_classes != all_classes:
            for klass in classes:
                class_name, parents = (
                    klass.name,
                    self.get_bases(klass),
                )
                skip = False
                for parent in parents:
                    self.imports_from_type(parent)
                    if "." not in parent and parent not in written_classes:
                        skip = True
                if class_name in written_classes:
                    skip = True

                if skip:
                    continue

                ordered_classes.append(klass)

                written_classes.add(class_name)

        return ordered_classes

    def extract_class(self, element: "_Element"):
        """Extract information from a class"""

        klass = ast.ClassDef()
        klass.name = element.attrib["name"]
        klass.bases = []
        klass.body = []
        klass.decorator_list = []

        docstring = get_doc(element)
        klass.body.append(ast.Expr(ast.Constant(docstring)))

        if parent := element.attrib.get("parent"):
            self.imports_from_type(parent)
            klass.bases.append(ast.alias(parent))

        implements = element.iterfind("implements", element.nsmap)

        for implement in implements:
            print(implement.nsmap)
            name = implement.attrib["name"]
            self.imports_from_type(name)
            klass.bases.append(ast.alias(name))

        methods = self.extract_methods(element)
        fields = self.extract_fields(element)

        klass.body.extend([*methods, *fields])

        return klass

    def insert_enum(self, element: "_Element"):

        """Returns an enum (class with attributes only) as text"""
        klass = ast.ClassDef()
        klass.name = element.attrib["name"]
        klass.bases = [ast.alias("enum.Enum")]
        klass.body = []
        klass.decorator_list = []

        docstring = get_doc(element)
        klass.body.append(ast.Expr(ast.Constant(docstring)))

        members = element.findall("{%s}member" % XMLNS)
        for member in members:
            const_name: str = member.attrib["name"]
            if not const_name:
                const_name = "_"
            if const_name and const_name[0].isdigit():
                const_name = "_" + const_name
            const_value: str = member.attrib["value"]
            const_value = const_value.replace("\\", "\\\\")
            if const_value == "(null)":
                const_value = 0
            klass.body.append(
                ast.Assign(
                    targets=[ast.Name(const_name.upper())],
                    value=ast.Constant(int(const_value)),
                )
            )

        return klass

    def analyze_namespace(self, namespace: "_Element"):
        """Extract all information from a gir namespace"""

        namespace_content = list[ast.stmt]()
        classes = list[ast.ClassDef]()

        for element in namespace:
            if element.tag in ("class", "interface", "record"):
                klass = self.extract_class(element)
                classes.append(klass)

            elif element.tag in ("enumeration", "bitfield"):
                self.gi_include.add("enum")
                namespace_content.append(self.insert_enum(element))

            elif element.tag == "function":
                params = self.get_parameters(element)
                returns = get_returntype(element)
                self.imports_from_type(returns)

                namespace_content.append(
                    self.make_function_ast(
                        element.attrib["name"],
                        params,
                        returns,
                        get_doc(element),
                    )
                )
            elif element.tag == "constant":
                constant_name = element.attrib["name"]
                if constant_name[0].isdigit():
                    constant_name = "_" + constant_name
                constant_value = element.attrib["value"] or ""
                constant_value = constant_value.replace("\\", "\\\\")
                namespace_content.append(
                    ast.Assign(
                        targets=[ast.Name(constant_name)],
                        value=ast.Constant(constant_value),
                    )
                )

        classes_content = self.build_classes(classes)
        namespace_content.extend(classes_content)

        for _import in self.gi_include:
            imp: ast.stmt

            if not _import:
                continue
            elif _import in ("typing", "enum"):
                imp = ast.Import([ast.alias(_import)])
            else:
                imp = ast.ImportFrom("gi.repository", [ast.alias(_import)], 0)
            namespace_content.insert(0, imp)

        return ast.Module(body=namespace_content)
