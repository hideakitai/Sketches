#include <iostream>

struct widget
{
    typedef int value_type;
};

template <class T>
void something (typename T::value_type*)
{
    std::cout << "I have value_type" << std::endl;
}

template <class T>
void something (...)
{
    std::cout << "others..." << std::endl;
}

int main()
{
    something<widget>(0); // something_of_widget(widget::value_type*) -> ok
    something<int>(0); // something_of_int(int::value_type*) -> error
}