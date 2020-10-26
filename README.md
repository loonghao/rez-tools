<p align="center">
<img src="https://i.imgur.com/oCFdRfj.png" alt="logo"></a>
</p>

<p align="center">
<a href="https://img.shields.io/pypi/pyversions/rez_tools">
<img src="https://img.shields.io/pypi/pyversions/rez_tools" alt="python version"></a>
<a href="https://badge.fury.io/py/rez_tools">
<img src="https://img.shields.io/pypi/v/rez_tools?color=green" alt="PyPI version"></a>
<img src="https://img.shields.io/pypi/dw/rez_tools" alt="Downloads Status"></a>
<a href="https://pepy.tech/badge/rez_tools">
<img src="https://pepy.tech/badge/rez_tools" alt="Downloads"></a>
<img src="https://img.shields.io/pypi/l/rez_tools" alt="License"></a>
<img src="https://img.shields.io/pypi/format/rez_tools" alt="pypi format"></a>
<a href="https://github.com/loonghao/rez_tools/graphs/commit-activity">
<img src="https://img.shields.io/badge/Maintained%3F-yes-green.svg" alt="Maintenance"></a>

</p>

<p align="center">
<strong><b>rez_tools</b></strong>
</p>


A suite tool command line for [rez](https://github.com/nerdvegas/rez).

<img src="https://i.imgur.com/rECBBUD.jpeg" alt="logo"></a>**This tool requires you to install [rez](https://github.com/nerdvegas/rez/wiki/Installation) in advance.**


Installing
----------
You can install via pip.

```cmd
pip install rez_tools
```

or through clone from Github.
```git exclude
git clone https://github.com/loonghao/rez-tools.git
```
Install package.
```cmd
cd rez_tools
```
```cmd
python setup.py install
```

QuickStart
----------
`rez_tools` will find all tools via `reztoolsconfig:tool_paths`.
```cmd
> set REZ_TOOL_CONFIG=C:\Users\hao.long\PycharmProjects\rez_tools\examples\reztoolsconfig.py
> rt
```
`rt` will dynamically generate a command line and bind it to the rt namespace 
based on the content defined in the found `.rt` file.

type `rt` will list all tools found. like the below.

```
Usage: rt [OPTIONS] PLUGIN [PLUGIN OPTIONS]

Options:
  -h, --help  Show this message and exit.

Plugin Commands:
  conan_python2  A rez plugin - conan_python2.
  conan_python3  A rez plugin - conan_python3.
  python         Python3.

Tool Options:
  rez_tools run other tools with their own options and argument patterns,
  however, all tool has the following hidden options:

  --ignore-cmd  Ignore standard tool command when running the command,Remember
                to provide an argument which will be used as the command to
                run.Examples: rt conan --ignore-cmd python

  --print       Print plugin details and exit.

```
---------------------------------------------

Define the suite description of `rez-tool`
------------------------------------------
The file format is .rt, the syntax is `yaml`

The following fields are now supported:


| Key         |required    | description                                |
|-------------|------------|------------------------------------------- |
| name        |    no      | The name of the tool, which will finally be registered in the command line.|
| command     |    yes     | The complete command line to be executed.  |
| requires    |    yes     | The name of the rez package that the command line execution environment depends on|

Examples:

**maya.rt**
```yaml
command: maya
requires:
   - maya-2020
   - maya_usd
   - maya_mtoa
```
run command line
```cmd
rt maya
```
-----------------------------------------------

**cmake_gui.rt**

```yaml
command: cmke-gui
requeres:
    - cmake
```
run command line
```cmd
rt cmake_gui
```

-----------------------------------------------

**python-2.rt**

```yaml
name: python_2
command: python
requeres:
    - python-2.7
    - pyside
    - pyyaml
```
run command line
```cmd
rt python_2
```