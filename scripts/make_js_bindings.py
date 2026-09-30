# `web/pxswrap.js` generator.
# Little comment by me Jordan, TODO


# Open library code
library = ""
with open("pixelscript.h", 'r') as f:
    library = f.read()


class Token:
    def __init__(self, line, col, type, string):
        self.line = line
        self.col = col
        self.type = type
        self.string = string

    def __str__(self):
        return str(self.__dict__())

    def __dict__(self):
        return {
            'line': self.line,
            'col': self.col,
            'type': self.type,
            'string': self.string
        }


TOKEN_BRACKET_LEFT = '['
TOKEN_BRACKET_RIGHT = ']'
TOKEN_PAREN_LEFT = '('
TOKEN_PAREN_RIGHT = ')'
TOKEN_TYPE = 'TYPE'
TOKEN_TYPEDEF = 'TYPEDEF'
TOKEN_STRUCT = 'STRUCT'
TOKEN_ENUM = 'ENUM'
TOKEN_FUNCTION = 'FUNC'
TOKEN_FUNCTION_PTR = 'FUNC_PTR'
TOKEN_EOF = 'EOF'
TOKEN_BRACE_LEFT = '{'
TOKEN_BRACE_RIGHT = '}'
TOKEN_COMMA = ','
TOKEN_SEMICOLON = ';'
TOKEN_MULTI_LINE_COMMENT = 'MULTI_LINE_COMMENT'
TOKEN_COMMENT = 'COMMENT'
TOKEN_PTR = 'PTR'
TOKEN_IDENT = 'IDENT'
TOKEN_INCLUDE = '#'
TOKEN_STRING = "STRING"
TOKEN_LT = "<"
TOKEN_GT = ">"
TOKEN_EQ = "="
TOKEN_NUMBER = "NUMBER"

class Lexer:
    def __init__(self, contents):
        self.contents = contents
        self.tokens = []
        self.c = 0
        self.is_eof = False
        self.line_number = 1
        self.col_number = 1
        self.char = ''

    def __str__(self):
        return f"{self.line_number}:{self.col_number} '{self.char}':'{self.peek_next_char()}'"

    def peek_next_char(self, start=None):
        """
        Peek to the next character. Skips new lines, tabs, winodws r lines.
        """
        if start == None:
            start = self.c
        if self.c >= len(self.contents) - 1:
            return ''
        ch = self.contents[start + 1]
        while ch in ['\n', '\t', '\r']:
            ch = self.peek_next_char(start + 1)
        return ch

    def parse_string(self):
        """
        Handles basic string parsing. The idea is that in pixelscript we dont expose any static strings soo yeah.
        It just goes from the first " to the next ". No escaping.
        """
        string = ""
        self.next_char()
        while self.char != '"':
            string += self.char
            self.next_char()
        self.push_token(TOKEN_STRING, string)

    def parse(self):
        """
        The magic parsing logic. Very basic stuff here really.
        """
        self.char = self.contents[self.c]
        while not self.is_eof:
            match self.char:
                case "=":
                    self.push_token(TOKEN_EQ)
                case '"':
                    self.parse_string()
                case "<":
                    self.push_token(TOKEN_LT)
                case ">":
                    self.push_token(TOKEN_GT)
                case '#':
                    self.next_char()
                    ident = self.get_ident()
                    self.push_token(TOKEN_INCLUDE, ident)
                case '[':
                    self.push_token(TOKEN_BRACKET_LEFT)
                case ']':
                    self.push_token(TOKEN_BRACKET_RIGHT)
                case '(':
                    self.push_token(TOKEN_PAREN_LEFT)
                case ')':
                    self.push_token(TOKEN_PAREN_RIGHT)
                case '{':
                    self.push_token(TOKEN_BRACE_LEFT)
                case '}':
                    self.push_token(TOKEN_BRACE_RIGHT)
                case ',':
                    self.push_token(TOKEN_COMMA)
                case ';':
                    self.push_token(TOKEN_SEMICOLON)
                case '*':
                    self.push_token(TOKEN_PTR)
                case '/':
                        if self.peek_next_char() == '*':
                            self.parse_multi_line_comment()
                            # Comments handle setting up the next char.
                            continue
                        elif self.peek_next_char() == '/':
                            self.parse_comment()
                            # Comments handle setting up the next char.
                            continue
                        else:
                            raise Exception("File is not formatted correctly (comments). " + str(self))
                case ' ':
                    pass
                case _:
                    if self.char in '0123456789':
                        self.get_number()
                    ident = self.get_ident()
                    if ident == 'typedef':
                        self.push_token(TOKEN_TYPEDEF, ident)
                    elif ident == 'enum':
                        self.push_token(TOKEN_ENUM, ident)
                    elif ident == 'struct':
                        self.push_token(TOKEN_STRUCT, ident)
                    else:
                        self.push_token(TOKEN_IDENT, ident)
                    continue

            self.next_char()
        self.push_token(TOKEN_EOF)

    def get_number(self):
        num = ""
        while True:
            num += self.char
            self.next_char()

            if self.char in '0123456789':
                continue
            else:
                break

        self.push_token(TOKEN_NUMBER, num)

    def get_ident(self) -> str:
        ident = ""
        while True:
            ident += self.char
            self.next_char()

            if self.char in 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890_':
                continue
            else:
                break

        return ident

    def parse_comment(self):
        comment = "/"
        self.next_char()
        line_number = self.line_number
        while line_number == self.line_number:
            comment += self.char
            self.next_char()
        self.push_token(TOKEN_COMMENT, comment)

    def parse_multi_line_comment(self):
        comment = '/*'
        # skip from '/' to '*'
        self.next_char()
        while not self.is_eof:
            comment += self.char
            if self.char == '*' and self.peek_next_char() == '/':
                self.next_char()
                self.next_char()
                break
            self.next_char()
        comment += '/'
        self.push_token(TOKEN_MULTI_LINE_COMMENT, comment)

    def push_token(self, type, string=None):
        if string == None:
            string = self.char
        self.tokens.append(Token(self.line_number, self.col_number, type, string))

    def next_char(self):
        self.c += 1

        if self.c >= len(self.contents):
            self.is_eof = True
            return
        self.char = self.contents[self.c]

        # Skip whitespace
        if self.char in ['\t', '\n', '\r']:
            if self.char == '\n':
                self.line_number += 1
                self.col_number = 1
            else:
                self.col_number += 1
            self.next_char()


class Transformer:
    def __init__(self, tokens):
        self.tokens = tokens
        self.definitions = ""
        self.bindings = ""
        self.i = -1
        self.is_eof = False
        self.token = None

    def next_token(self):
        self.i += 1
        if self.i >= len(self.tokens):
            self.is_eof = True
            return

        self.token = self.tokens[self.i]

    def peek_token(self):
        if self.i >= len(self.tokens) - 1:
            return Token(0, 0, "", "")
        return self.tokens[self.i + 1]

    def peek_token_is(self, token_type) -> bool:
        return self.peek_token().type == token_type

    def error(self, msg):
        raise Exception(f"{msg}: {self.tokens[self.i - 1]} {self.token}, {self.peek_token()}")

    def next_expect(self, type):
        if not self.peek_token_is(type):
            self.error('Expected type: "' + type + '"')
            exit(1)
        self.next_token()

    def while_token(self, type):
        return not self.token.type == type and not self.is_eof

    def write_enum(self):
        self.next_token()
        code = ["const"]
        self.next_expect(TOKEN_IDENT)
        code.append(self.token.string)
        self.next_expect(TOKEN_BRACE_LEFT)
        code.append("={")
        num_idents = 0
        while self.while_token(TOKEN_BRACE_RIGHT):
            self.next_token()
            if self.token.type == TOKEN_IDENT:
                code.append(self.token.string)
                if self.peek_token_is(TOKEN_EQ):
                    self.next_token()
                    self.next_token()
                    code.append(":")
                    code.append(self.token.string)
                    code.append(",")
                    self.next_token()
                else:
                    code.append(":")
                    code.append(str(num_idents))
                    num_idents += 1
            if self.token.type == TOKEN_COMMA:
                code.append(",")
        code.append("};\n")
        self.definitions += " ".join(code)

    def write_struct(self):
        self.next_token()
        self.next_expect(TOKEN_IDENT)
        if self.peek_token_is(TOKEN_BRACE_LEFT):
            while self.while_token(TOKEN_BRACE_RIGHT):
                self.next_token()
            self.next_token() # }
            self.next_token() # name
            self.next_token() # semicolon.
        elif self.peek_token_is(TOKEN_IDENT):
            self.next_token() # name
            if self.peek_token_is(TOKEN_PTR):
                self.next_token() #*
            self.next_token() # semicolon

    def parse_function_ptr_type(self, string) -> str:
        if string.endswith('T') or string == 'pxs_Opaque':
            return 'p'
        if string == 'void':
            return 'v'
        return None

    def write_function_ptr(self):
        code = ["const"]
        self.next_token()
        if self.token.string == 'union':
            return
        if not self.peek_token_is(TOKEN_PAREN_LEFT):
            return
        type = "'"
        
        self.next_expect(TOKEN_PAREN_LEFT)
        self.next_expect(TOKEN_PTR)
        self.next_expect(TOKEN_IDENT)

        code.append(self.token.string)
        code.append("=")

        type += self.parse_function_ptr_type(self.token.string) or 'v'

        self.next_expect(TOKEN_PAREN_RIGHT)
        self.next_expect(TOKEN_PAREN_LEFT)
        while self.while_token(TOKEN_PAREN_RIGHT):
            self.next_token()
            sent = self.token.string
            ftype = self.parse_function_ptr_type(self.token.string)
            if not ftype is None:
                print(f"sent: {sent} got {ftype}")
                type += ftype
            self.next_token()
            if self.peek_token_is(TOKEN_COMMA):
                self.next_token()
        code.append(f"{type}'")
        code.append(";\n")
        self.definitions += ' '.join(code)
# typedef pxs_VarT (*pxs_LoadFileFn)(const char *file_path);

    def write(self):
        self.next_token()

        while not self.is_eof:
            if self.token.type == TOKEN_TYPEDEF:
                if self.peek_token().type == TOKEN_ENUM:
                    self.write_enum()
                elif self.peek_token_is(TOKEN_STRUCT):
                    self.write_struct()
                elif self.peek_token_is(TOKEN_IDENT):
                    self.write_function_ptr()
            self.next_token()

lex = Lexer(library)
lex.parse()
t = Transformer(lex.tokens)
t.write()
print(t.definitions)

# for token in lex.tokens:
    # print(token)
