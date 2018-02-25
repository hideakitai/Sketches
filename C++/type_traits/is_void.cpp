#include <iostream>

template <class T>
struct is_void
{
    static const bool value = false;
};

template <>
struct is_void <void>
{
    static const bool value = true;
};

int main()
{
    std::cout << std::boolalpha;
    std::cout << is_void<void>::value << std::endl;
    std::cout << is_void<int>::value << std::endl;
    std::cout << is_void<char*>::value << std::endl;
}