from setuptools import find_packages
from setuptools import setup

setup(
    name='rez-tools',
    version='0.1.62',
    package_dir={"": "."},
    packages=find_packages("."),
    url='',
    license='MTL',
    author='Long Hao',
    author_email='hal.long@outlook.com',
    entry_points={
        "console_scripts": [
            "rt = rez_tools.__main__:main",
        ]
    },
    description='',
    # use_scm_version=True,
    setup_requires=["setuptools_scm"],
)