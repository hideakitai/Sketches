# distutils: language = c++
# distutils: sources = ../Hello/Hello.cpp

# C++ class interface

cdef extern from '../Hello/Hello.h' namespace 'HelloCy':
    cdef cppclass Hello:
        Hello(int) except +
        void say()
        int id


# Cython wrapper class

cdef class HelloPy:
    cdef Hello *thisptr

    def __cinit__(self, id):
        self.thisptr = new Hello(id)

    def __dealloc__(self):
        del self.thisptr

    def say(self):
        self.thisptr.say()
