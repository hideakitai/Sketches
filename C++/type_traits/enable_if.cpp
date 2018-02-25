#include <iostream>
#include <boost/utility.hpp>
#include <type_traits>

template <class T, typename std::enable_if<std::is_class<T>::value>::type* = nullptr>
void check(T value)
{
    std::cout << "T is class" << std::endl;
};

template <class T, typename std::enable_if<!std::is_class<T>::value>::type* = nullptr>
void check(T value)
{
    std::cout << "T is not class" << std::endl;
};


struct X {};

int main()
{
    check(X());
    check(3);
}