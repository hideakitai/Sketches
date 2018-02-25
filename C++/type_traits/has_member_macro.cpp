#include <iostream>
#include <type_traits>

struct X { int id; };
struct Y { int foo; };

#define DEFINE_MEMBER_CHECKER(member) \
    template<typename T, typename V = bool> \
    struct has_ ## member : std::false_type { }; \
    template<typename T> \
    struct has_ ## member<T, \
        typename std::enable_if< \
            !std::is_same<decltype(std::declval<T>().member), void>::value, \
            bool \
            >::type \
        > : std::true_type { };

#define HAS_MEMBER(C, member) \
    has_ ## member<C>::value

DEFINE_MEMBER_CHECKER(foo)

int main()
{
    std::cout << std::boolalpha;
    std::cout << HAS_MEMBER(X, foo) << std::endl;
    std::cout << HAS_MEMBER(Y, foo) << std::endl;
}
