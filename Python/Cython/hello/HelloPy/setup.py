from distutils.core import setup, Extension
from Cython.Distutils import build_ext
from Cython.Build import cythonize

ext = Extension(
    "HelloPy",                             # name of extension
    ["HelloPy.pyx", "../Hello/Hello.cpp"], # filename of our Pyrex/Cython source
    language="c++",                        # this causes Pyrex/Cython to create C++ source
    include_dirs=[],
    libraries=[],                          # ditto
    extra_compile_args=['-std=c++11'],
    extra_link_args=['-std=c++11'],
    cmdclass = {'build_ext': build_ext}
)

setup(name = "HelloPy", ext_modules = cythonize([ext]))
