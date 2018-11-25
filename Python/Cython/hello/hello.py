import sys
sys.path.append('HelloPy')

from HelloPy import HelloPy

hello = HelloPy(3)
hello.say()
# hello from Cython! I am 3
