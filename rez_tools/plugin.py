import os
import subprocess
from yaml import load, FullLoader

from rez_tools import reztoolsconfig


class Plugin(object):
    def __init__(self, file_path):
        self._path = file_path
        self._data = self._read()

    def _read(self):
        with open(self._path, "rb") as file_obj:
            return load(file_obj, Loader=FullLoader)

    @property
    def rez_opts(self):
        return []

    @property
    def run_detached(self):
        return self._data.get("run_detached", False)

    @property
    def path(self):
        return self._path

    @property
    def short_help(self):
        return self._data.get("short_help",
                              "A rez plugin - {}.".format(self.name))

    @property
    def name(self):
        """str: name of tool, Default from file name."""
        basename = self._data.get("name")
        if not basename:
            basename = os.path.basename(self.path)
            basename = basename.split(reztoolsconfig.extension)[0]
        return basename

    @property
    def command(self):
        """str: Command line to be executed."""
        return self._data["command"]

    @property
    def inherits_from(self):
        """The tools inherits from."""
        return self._data.get("inherits_from")

    @property
    def packages(self):
        """All requires package from .rt file."""
        return self._data["requires"]

    def assemble_command(self, command=None):
        """str: Assemble command line for rez-env."""
        rez_command = ['rez', 'env', '-q']

        # Add the packages.
        rez_command.extend(self.packages)

        # add whatever command the user is passing in to the rez call
        rez_command.append('--')
        if not command:
            rez_command.append(self.command)
        else:
            rez_command.extend(command)
        return subprocess.list2cmdline(rez_command)

    def as_dict(self):
        return self._data

    @staticmethod
    def run(command, detached=False):
        """Launch a non-interactive command in a prepared contextual environment.

        Args:
            command (str): The complete command line to be executed.
            detached (bool): If True, run the command in a new, detached
                terminal and exit pxo immediately. If False (default), run
                cmd and wait for it to exit.

        Returns:
            int: Return code of the process run.
        """
        kwargs = {
            'shell': True,
            'env': os.environ.copy(),
            'close_fds': True,
        }
        if detached:
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            startupinfo.wShowWindow = 3

            kwargs.update({
                'creationflags': subprocess.CREATE_NEW_CONSOLE,
                'startupinfo': startupinfo,
            })
            return subprocess.Popen('START /W ' + command, **kwargs).returncode
        else:
            return subprocess.call(command, **kwargs)
