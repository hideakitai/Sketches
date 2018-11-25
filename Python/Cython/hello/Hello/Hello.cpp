#include <iostream>
#include "Hello.h"

using namespace HelloCy;

Hello::Hello(int i) : id(i)
{
}

void Hello::say()
{
    std::cout << "hello from Cython! I am " << id << std::endl;
}
