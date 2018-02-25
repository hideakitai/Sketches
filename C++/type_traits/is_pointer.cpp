#include <iostream>

template <class T>
struct is_pointer
{
    static const bool value = false;
};

template <class T>
struct is_pointer<T*>
{
    static const bool value = true;
};

int main()
{
    std::cout << std::boolalpha;
    std::cout << is_pointer<void>::value << std::endl;
    std::cout << is_pointer<int>::value << std::endl;
    std::cout << is_pointer<char*>::value << std::endl;
}
