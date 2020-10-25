import os

# `rt` will search all tools from the below paths.
tool_paths = [
    os.path.normpath(os.path.expanduser("~/packages")),
    os.path.dirname(__file__)
]

extension = ".rt"
