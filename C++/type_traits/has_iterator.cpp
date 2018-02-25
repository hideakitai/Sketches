#include <iostream>

template <class T>
class has_iterator
{
    template <class U>
    static constexpr bool check (typename U::iterator*) { return true; }

    template <class U>
    static constexpr bool check (...) { return false; }

public:
    static const bool value = check<T>(nullptr);
};


#include <vector>

int main()
{
    static_assert(has_iterator<std::vector<int>>::value, "vector has iterator");
    static_assert(!has_iterator<int>::value, "int doesn't have iterator");
}