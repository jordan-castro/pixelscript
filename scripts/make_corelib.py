# Generate the files for the corelib.

from dataclasses import dataclass


# The files that are used for corelib
corelib = {
    "pxs.http": "src/pxs_core/http/mod.rs",
    "pxs.mem": "src/pxs_core/pxs_mem.rs",
    "pxs.fs": "src/pxs_core/pxs_fs.rs",
    "pxs.os": "src/pxs_core/pxs_os.rs",
    "pxs_shell": "src/pxs_core/pxs_shell.rs"
}

STARTING_CODE = """
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PixelScript docs</title>

    <script src="/pxswrap.js"></script>
    <script src="/components/bootstrap.js"></script>
    <link rel="stylesheet" href="/styles/terminal_line.css">
    <link href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism.min.css" rel="stylesheet">

    <style>
        body {
            background-color: black;
            color: white;
        }

        html {
            scroll-behavior: smooth;
        }
    </style>

    <link rel="stylesheet" href="https://unpkg.com/dracula-prism/dist/css/dracula-prism.css">
    <script>
        const oglog = console.log;
        let CURRENT_ID = "";

        function clear_output() {
            const output = document.getElementById(CURRENT_ID);
            if (!output) {
                console.error("Can not clear null output.");
                return;
            }
            output.replaceChildren();
        }

        function write_output(...args) {
            let output = document.getElementById(CURRENT_ID);
            if (!output) {
                return;
            }
            for (let arg of args) {
                let div = document.createElement("div");
                div.className = "code-result-text";
                div.innerHTML = arg;
                output.appendChild(div);
            }
        }

        console.log = (...args) => {
            oglog(...args);
            write_output(args);
        };

        const CODES = {{$CODES$}};
        function runblock(cid) {
            clear_output();
            CURRENT_ID = cid;
            clear_output();
            if (CODES[cid]) {
                let val = pxs_exec(pxs_Runtime.pxs_Python, CODES[cid], cid);
                if (pxs_isexception(val)) {
                    console.log(pxs_getstring(val));
                }
                pxs_freevar(val);
            }
        }
    </script>
</head>

<body>
    <div class="container">
        <div id="pxs-navbar"></div>
        <h2>PXS Docs</h2>
        <hr>
        <h3>CoreLib</h3>
        <p>Pixelscript comes with an optional core library modules. They sit behind the <span
                class="terminal-line">pxs</span> module.</p>
        <p>To use the core library you must run <span class="terminal-line">pxs_core_initall()</span>. Or you can
            initialize specific modules.</p>
        <hr>
        <section id="specific-modules">
            <a class="clean-anchor" href="#specific-modules">
                <h5>Specific Modules</h5>
            </a>
        </section>

        <p>There are specific modules in pixelscript that can be added or not.</p>
        <table class="table table-dark">
            <thead>
                <tr>
                    <th scope="col">Module</th>
                    <th scope="col">Enum</th>
                    <th scope="col">Value</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <th scope="row">pxs.mem</th>
                    <th>pxs_ModuleFlag::pxs_MEM</th>
                    <th>2</th>
                </tr>
                <tr>
                    <th scope="row">pxs.os</th>
                    <th>pxs_ModuleFlag::pxs_OS</th>
                    <th>4</th>
                </tr>
                <tr>
                    <th scope="row">pxs</th>
                    <th>pxs_ModuleFlag::pxs_PXS</th>
                    <th>8</th>
                </tr>
                <tr>
                    <th scope="row">pxs.fs</th>
                    <th>pxs_ModuleFlag::pxs_FS</th>
                    <th>16</th>
                </tr>
                <tr>
                    <th scope="row">pxs.shell</th>
                    <th>pxs_ModuleFlag::pxs_SHELL</th>
                    <th>32</th>
                </tr>
                <tr>
                    <th scope="row">pxs.zip</th>
                    <th>pxs_ModuleFlag::pxs_ZIP</th>
                    <th>64</th>
                </tr>
                <tr>
                    <th scope="row">pxs.http</th>
                    <th>pxs_ModuleFlag::pxs_HTTP</th>
                    <th>128</th>
                </tr>
            </tbody>
        </table>
        <hr>
        {$CORE_LIB_PLACEMENT$}
    </div>
    <script>
        var Module = {
            onRuntimeInitialized: function () {
                console.log("PXS is ready.");
                pxs_wrap(Module);
                READY = true;
                pxs_core_initall();
            }
        };
    </script>

    <script src="/components/navbar.js"></script>
    <script src="/pixelscript.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/prism.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-python.min.js"></script>
</body>

</html>
"""


class CoreLibWriter:
    def __init__(self):
        self.lines = []
        self.pos = -1
        self.line = ""
        self.finish = False
        self.values = []

    def parse_type(self, value):
        type_comment = value.split("(")
        value = type_comment[0]
        comment = None
        if len(type_comment) == 2:
            comment = type_comment[1]
        return {
            'value': type_comment[0],
            'comment': comment
        }

    def parse_values(self):
        # /// @values(Read = 1 << 0, Write = 1 << 1, Append = 1 << 2).
        values = []
        line = self.line.replace("/// @values(", "")
        raw_values = line.split(",")
        for rvalue in raw_values:
            name_type = rvalue.split("=")
            value = {
                'name': name_type[0],
                'type': name_type[1]
            }
            values.append(value)
        return values

    def parse_class_method(self, value=None):
    #     /// @classmethod #`fs-File`
        if value == None:
            value = self.line
        line = value.replace("/// @classmethod", "")
        value = None
        if line[0] == '(':
            value = line[1:-1]
        value = line[2:-1]
        return value.replace('`', '')

    def parse_return(self):
    #     /// @returns(#`fs-File`) some comment
        line = self.line.replace("/// @return(", "")
        type = line[0:line.find(')')]
        if type[0] == '#':
            type = self.parse_class_method(type)
        # type = type[2:-1]
        comment = None
        if ')' in line:
            comment= line.split(")")[-1]
        return {
            'type': type,
            'comment': comment
        }

    def parse_args(self):
        # @args(name:type(comment))
        args = []
        line = self.line.replace("/// @args(", "")
        raw_args = line.split(",")
        for rarg in raw_args:
            name_type = rarg.split(":")
            arg = {
                'name': name_type[0],
                'type': self.parse_type(name_type[1])
            }
            args.append(arg)
        return args

    def parse_prop(self):
        # @prop(get,set) name
        line = self.line.replace("/// @prop", "")
        types = ['get', 'set']
        name = ''
        if line[0] == '(':
            types = line.split(")")[0].split(",")
            name = line.split(")")[-1]
        else:
            name = line

        return {
            'types': types,
            'name': name
        }

    def parse_example(self):
        example = []
        while True:
            self.next_line()
            if '@example(end)' in self.line:
                break
            example.append(self.line.replace('/// ', '').replace('///', ''))
        return example

    def parse_pxs(self):
        value = {}
        if '(' in self.line:
            value['name'] = self.line.split('(')[-1][:-1]
        while not self.finish:
            self.next_line()
            if not '///' in self.line:
                if 'extern "C"' in self.line and not value.get('name', None):
                    value['name'] = self.line.split('(')[0].replace('extern "C" fn ', '')
                if not len(value.keys()) == 0:
                    self.values.append(value)
                break
            if '@enum' in self.line:
                value['is_enum'] = True
            elif '@args' in self.line:
                value['args'] = self.parse_args()
            elif '@values' in self.line:
                value['values'] = self.parse_values()
            elif '@self' in self.line:
                value['needs_self'] = True
            elif '@classmethod' in self.line:
                value['parent'] = self.parse_class_method()
            elif '@return' in self.line:
                value['returns'] = self.parse_return()
            elif '@prop' in self.line:
                value['needs_self'] = True
                value['property'] = self.parse_prop()
            elif '@example' in self.line:
                value['example'] = self.parse_example()
            elif '@except' in self.line:
                value['excepts'] = True
            else:
                if value.get('comments', None) == None:
                    value['comments'] = []
                value['comments'].append(self.line.replace('///', ''))

    def next_line(self):
        self.pos += 1
        if self.pos >= len(self.lines):
            self.finish = True
            return
        self.line = self.lines[self.pos].strip()

    def parse(self, file):
        self.lines = []
        self.pos = -1
        with open(file, 'r') as f:
            self.lines = f.readlines()
        while not self.finish:
            self.next_line()
            if '/// @pxs' in self.line:
                self.parse_pxs()


final_contents = ""
classes = []
codes = ""


def clean_anchor(contents):
    return contents.replace(",", "").replace("-", "").replace("`", "").replace(".", "").strip()

for f in corelib:
    writer = CoreLibWriter()
    writer.parse(corelib[f])
    final_contents += f"""
        <hr>
        <section id="{f}">
            <a class="clean-anchor" href="#{f}">
                <h5>{f}</h5>
            </a>
        </section>
    """

    for value in writer.values:
        if value.get('parent', None):
            if not value['parent'] in classes:
                classes.append(value['parent'])
                final_contents += f"""
                <section>
                    <a class="clean-anchor" href="#{clean_anchor(
                        f + value['parent'].split('-')[1]
                    )}">
                        <h6>{value['parent'].split('-')[1]}</h6>
                    </a>
                </section>
                """
        if value.get('property', None):
            prop = value['property']
            final_contents += f"""
            <section>
            <a class="clean-anchor" href="#{value['parent'].strip()}{prop['name'].strip()}"><h7>{prop['name']}</h7></a>
            </section>
            <div class="pxs-property{' get' if 'get' in prop['types'] else ''}{' set' if 'set' in prop['types'] else ''}"></div>
            """
        else:
            final_contents += f"""
            <section>
            <a class="clean-anchor" href="#{value['parent'] if 'parent' in value else ''}{value['name']}"><h7>{value['name']}</h7></a>
            </section>
            """
        if value.get('is_enum', False):
            final_contents += '<div class="pxs-enum"></div>'
        if value.get('needs_self', False):
            final_contents += '<div class="pxs-self"></div>'
        if value.get('excepts', False):
            final_contents += '<div class="pxs-excepts"></div>'
        if value.get('args', None):
            args = value['args']
            string = []
            for arg in args:
                #         <p>Paramaters are: <span class="terminal-line">object: pxs_Object</span></p>
                string.append(f"{arg['name']}: {arg['type']['value']} {arg['type']['comment']}.")
            final_contents += f"<p>Paramaters are: {' '.join(string)} </p>"
        if value.get('returns', None):
            returns = value['returns']
            final_contents += f"<p>Returns: {returns['type']} {returns['comment']}</p>"
        if value.get('comments', None):
            for comment in value['comments']:
                final_contents += f'<p>{comment}</p>'
        if value.get('example', None):
            example = value['example']
            final_contents += f"""
            <div class="row">
            <div class="col">
            <pre class="language-python code-container">
                <code class="code-editor">
{'\n'.join(example)}
                </code>
            </pre>          
            </div>
            <div class="col">
                <button type="button" class="btn btn-success" onclick="runblock('{value['name']}')">Try it out</button>
                <div class="code-container" id="{value['name']}">
                </div>
            </div>
            </div>  
"""
            codes += f"{value['name']}: `{'\n'.join(example)}`,"

code = STARTING_CODE.replace("{$CORE_LIB_PLACEMENT$}", final_contents)
code = code.replace("{$CODES$}", codes)
with open("web/docs/corelib.html", "w") as f:
    f.write(code)