# The tool template used when building runnable tool Commands.
TOOL_TEMPLATE = """
@{group.name}.command(
    name='{plugin.name}',
    short_help='{plugin.short_help}',
    add_help_option=False,
    context_settings={{
        'ignore_unknown_options': True,
        'allow_interspersed_args': False}})
@click.option(
    '--use-standard-cmd/--ignore-standard-cmd',
    default=True,
    is_eager=True)
@click.option(
    '--print-plugin-details',
    is_flag=True,
    is_eager=True)
@click.option(
    '--run-plugin-detached',
    is_flag=True)
@click.option(
    '--force-rez-env-time',
    type=str)
@click.argument('args', nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def _{plugin.name}(ctx, use_standard_cmd,
                   run_plugin_detached, args,
                   print_plugin_details,
                   force_rez_env_time):
    if print_plugin_details:
        click.echon(json.dumps({plugin_dict}, indent=4))
        ctx.exit()
    rez_opts = {plugin.rez_opts} or {{}}
    if force_rez_env_time:
        rez_opts['time'] = force_rez_env_time
    detached = {plugin.run_detached} or run_plugin_detached
    PLUGIN.run(detached)
"""
