from setuptools import find_packages
from setuptools import setup

with open("README.md", "r") as fh:
    long_description = fh.read()

setup(
    name='rez-tools',
    package_dir={"": "."},
    packages=find_packages("."),
    url='https://github.com/loonghao/rez-tools',
    license='MTL',
    author='Long Hao',
    author_email='hal.long@outlook.com',
    description="A suite tool command line for rez.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    entry_points={
        "console_scripts": [
            "rt = rez_tools.__main__:main",
        ]
    },
    use_scm_version=True,
    setup_requires=['setuptools_scm'],
)
