import os

from rez_tools import reztoolsconfig
from rez_tools.cli import cli

UserError = type("UserError", (Exception,), {})


def _load_config(file_path=None):
    """Load config.

    Args:
        file_path (str): Absolute file path of the config.

    Returns:
        str: Absolute file path of the config.

    References:
        https://github.com/mottosso/allzpark/blob/405d25052191c4fbc68d52f23aec3bd8034861ec/allzpark/cli.py#L19

    """
    file_path = file_path or os.getenv("REZ_TOOL_CONFIG",
                                       os.path.expanduser(
                                           "~/reztoolsconfig.py"))
    if not os.path.isfile(file_path):
        return

    mod = {
        "__file__": file_path,
    }

    try:
        with open(file_path) as file_obj:
            exec(compile(file_obj.read(), file_obj.name, "exec"), mod)
    except IOError:
        raise

    except Exception:
        raise UserError("Better double-check your user config.")

    for key in dir(reztoolsconfig):
        if key.startswith("__"):
            continue

        try:
            value = mod[key]
        except KeyError:
            continue
        setattr(reztoolsconfig, key, value)

    return file_path


def _patch_reztoolsconfig():
    """Make backup copies of originals, with `_` prefix

    Useful for augmenting an existing value with your own config

    """
    for member in dir(reztoolsconfig):
        if member.startswith("__"):
            continue

        setattr(reztoolsconfig, "_%s" % member,
                getattr(reztoolsconfig, member))


def main():
    _patch_reztoolsconfig()
    _load_config()
    cli()
