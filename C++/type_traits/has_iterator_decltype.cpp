#include <iostream>
#include <type_traits>

// std::true_type  has the member static constexpr bool value = true;
// std::false_type has the member static constexpr bool value = flase;
struct has_iterator_impl
{
    template <class T>
    static std::true_type check(typename T::iterator*);

    template <class T>
    static std::false_type check(...);
};

// decltype does not call function,
// so we need just the definition of interface (return, func_name, input)
template <class T>
class has_iterator : public decltype(has_iterator_impl::check<T>(nullptr)) {};


#include <vector>

int main()
{
    static_assert(has_iterator<std::vector<int>>::value, "vector has iterator");
    static_assert(!has_iterator<int>::value, "vector has iterator");
}