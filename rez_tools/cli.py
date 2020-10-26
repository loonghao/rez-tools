import json
import logging
import os
import sys
from collections import OrderedDict
from glob import iglob
import re

import click
from yaml import YAMLError

from rez_tools import reztoolsconfig
from rez_tools.plugin import Plugin
from rez_tools.template import TOOL_TEMPLATE


class ToolGroup(click.Group):
    plugins = None
    plugin_template: str = TOOL_TEMPLATE
    plugin_command_metavar = 'PLUGIN [PLUGIN OPTIONS]'

    def __init__(self, name=None, commands=None, **attrs):
        if attrs.get('subcommand_metavar') is None:
            attrs['subcommand_metavar'] = self.plugin_command_metavar
        super(ToolGroup, self).__init__(name, commands, **attrs)

    @staticmethod
    def get_tools():
        """Walk the given paths looking for .rt.

        Returns:
            dict: A dictionary mapping of the plugins and their properties.

        """
        logger = logging.getLogger(__name__)
        plugins = []
        for path in reversed(reztoolsconfig.tool_paths):
            inheriting_plugins = []
            for plugin_file_path in iglob(os.path.join(path, '*' + reztoolsconfig.extension)):
                try:
                    plugin = Plugin(plugin_file_path)
                    pattern = '^[a-zA-Z][a-zA-Z0-9_]+$'
                    verify_name = re.match(pattern, plugin.name)
                    if not verify_name:
                        logger.warning("The name of the plug-in does not match, please modify it to a name that "
                                       "conforms to the rules: %s\n %s", pattern, plugin_file_path)
                        continue
                    if plugin.inherits_from:
                        logger.debug('Deferring load of sub-plugin {0}'.format(plugin.name))
                        inheriting_plugins.append(plugin)
                        continue

                except OSError as err:
                    logger.warning('Unable to read plugin {0}: {1}'.format(plugin_file_path, err))
                    continue
                except YAMLError as err:
                    logger.warning('Unable to parse plugin {0}: {1}'.format(plugin_file_path, err))
                    continue
                except ValueError as err:
                    logger.warning('Unable to validate plugin {0}: {1}'.format(plugin_file_path, err))
                    continue
                plugins.append(plugin)

        return plugins

    def populate_plugins(self, ctx):
        """Populate the class variable that holds all the dynamically-loaded

        plugins. Run lazily the first time list_plugins or get_command are called.

        Args:
            ctx (click.Context): The current context.

        Returns:
            dict: the plugins dict.

        """
        # lazily load the plugins list
        if self.plugins is None:
            plugins = self.get_tools()
            self.plugins = {
                _plugin.name: _plugin
                for _plugin in plugins
            }
        return self.plugins

    def get_namespace(self, class_name):
        """Get the namespace for plugin compile. Subclasses will need to add to this.

        Args:
            class_name (str): The name of the class being compiled.

        Returns:
            dict: The namespace for compiling the plugin.

        """
        return {
            "__name__": '{0.__class__.__name__}_get_command_{1}'.format(self, class_name),
            "click": click,
            "OrderedDict": OrderedDict,
            "json": json,
            "ToolGroup": ToolGroup,
            self.name: self,
            "PLUGIN": self.plugins[class_name[1:]]
        }

    def get_command(self, ctx, cmd_name):
        """As per click.Group.get_command, but populates the plugin list.

        Args:
            ctx (click.Context): The current context.
            cmd_name (str): The command being run.

        Returns:
            click.Command: The Command instance to run.

        """
        cmd = super(ToolGroup, self).get_command(ctx, cmd_name)
        # We may be looking for a plugin, which we'll compile only as required
        if cmd is None:
            cmd = self.populate_plugins(ctx).get(cmd_name)
            if cmd is not None:
                class_name = '_{}'.format(cmd_name)
                class_definition = self.plugin_template.format(group=self,
                                                               plugin=cmd,
                                                               plugin_dict=cmd.as_dict())
                namespace = self.get_namespace(class_name)
                try:
                    # pylint: disable=exec-used
                    exec(class_definition, namespace)
                except SyntaxError as error:
                    raise SyntaxError('{}:\n{}'.format(error,
                                                       class_definition))
                cmd = namespace[class_name]
        return cmd

    @staticmethod
    def should_list(plugin):
        """Filter function to determine if a plugin should be listed.
        Subclasses should overload with specific behaviors.

        Args:
            plugin (namedtuple): The plugin to filter.

        Returns:
            bool

        """
        return bool(plugin)

    def list_plugins(self, ctx):
        """Returns the sorted list of plugin names.

        Args:
            ctx (click.Context): The current context.

        Returns:
            list: list of strings
        """
        result = []
        for cmd_name, plugin in sorted(self.populate_plugins(ctx).items()):
            if self.should_list(plugin):
                result.append(cmd_name)
        return result

    def format_plugins(self, ctx, formatter):
        """Format the plugins for this PluginGroup.

        Args:
            ctx (click.Context): The current context.
            formatter (click.Formatter): The current formatter.

        Returns:
            bool

        """
        rows = []
        plugins = self.list_plugins(ctx)

        for plugin in plugins:
            cmd = self.get_command(ctx, plugin)
            if cmd is None:
                continue
            if getattr(cmd, 'hidden', False):
                continue
            rows.append((plugin, cmd.short_help or ''))

        if rows:
            with formatter.section('Plugin Commands'):
                formatter.write_dl(rows)
            return True
        return False

    def format_plugin_options(self, ctx, formatter):
        """Format the plugin options for this PluginGroup.

        Args:
            ctx (click.Context): The current context.
            formatter (click.Formatter): The current formatter.

        """
        head = "rez-tools run other tools with their own options and argument patterns, " \
               "however, all tool has the following hidden options: "
        opts = [
            (
                '--ignore-cmd',
                'Ignore standard tool command when running the command,'
                'Remember to provide an argument which will be used as the command '
                'to run.'
                'Examples: rt conan --ignore-cmd python',
            ),
            (
                '--print',
                "Print plugin details and exit."
            )
        ]
        with formatter.section("Tool Options"):
            formatter.write_text(head)
            formatter.write_paragraph()
            formatter.write_dl(opts)

    def parse_args(self, ctx, args):
        if not args and self.no_args_is_help:
            if not ctx.resilient_parsing:
                click.echo(ctx.get_help(), color=ctx.color)
                ctx.exit()

        parser = self.make_parser(ctx)
        passed_opts, passed_args, passed_order = parser.parse_args(args=args[:])

        passed_options = OrderedDict()
        for opt in passed_order:
            if opt.name not in passed_options:
                passed_options[opt.name] = opt

        all_options = OrderedDict()
        for opt in self.get_params(ctx):
            if opt.name not in all_options:
                all_options[opt.name] = opt
        return super(ToolGroup, self).parse_args(ctx, passed_args)

    def format_options(self, ctx, formatter):
        """Format the options and add the plugin details (if any) to the
        message.

        Args:
            ctx (click.Context): The current context.
            formatter (click.Formatter): The current formatter.

        """
        super(ToolGroup, self).format_options(ctx, formatter)
        if self.format_plugins(ctx, formatter):
            self.format_plugin_options(ctx, formatter)


@click.command(
    name='rez_tools',
    cls=ToolGroup,
    invoke_without_command=False,
    context_settings={'help_option_names': ['-h', '--help']})
def cli():
    pass
