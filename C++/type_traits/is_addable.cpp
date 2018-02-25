#include <iostream>
#include <utility>

struct is_addable_impl
{
    template <class T, class U>
    static auto check(T*, U*) -> decltype(std::declval<T>() + std::declval<U>(), std::true_type());

    template <class T, class U>
    static auto check(...) -> std::false_type;
};

template <class T, class U>
struct is_addable : decltype(is_addable_impl::check<T, U>(nullptr, nullptr)) {};

struct A {};
struct B {};
B operator+ (const B&, const B&) { return B(); }

int main()
{
    std::cout << std::boolalpha;
    std::cout << is_addable<A, A>::value << std::endl;
    std::cout << is_addable<B, B>::value << std::endl;
    std::cout << is_addable<int, double>::value << std::endl;
}