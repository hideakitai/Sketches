#include <type_traits>

struct is_class_impl
{
    template <class T>
    static std::true_type check(int T::*); // any pointer type

    template <class T>
    static std::false_type check(...);
};

template <class T>
struct is_class : public decltype(is_class_impl::check<T>(nullptr)) {};

struct X {};

int main()
{
    static_assert(is_class<X>::value, "X is class");
    static_assert(!is_class<int>::value, "int is not class");
}