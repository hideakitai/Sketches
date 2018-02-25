#include <iostream>
#include <type_traits>

struct X { int id; void test(){} };
struct Y { int foo; void check(){} };

#define DEFINE_MEMBER_CHECKER(member) \
    template<typename T, typename V = bool> \
    struct has_ ## member : std::false_type { }; \
    template<typename T> \
    struct has_ ## member<T, \
        typename std::enable_if< \
            !std::is_same<decltype(std::declval<T>().member), std::false_type>::value, \
            bool \
            >::type \
        > : std::true_type { };

#define HAS_MEMBER(C, member) \
    has_ ## member<C>::value

#define DEFINE_FUNCTION_CHECKER(function) \
    template<typename T, typename V = bool> \
    struct has_ ## function : std::false_type { }; \
    template<typename T> \
    struct has_ ## function<T, \
        typename std::enable_if< \
            !std::is_same< \
                decltype(std::declval<T>().function()), \
                std::false_type \
            >::value, \
            bool \
        >::type \
    > : std::true_type { };

#define HAS_FUNCTION(C, function) \
    has_ ## function<C>::value
#define HAS_FUNCTION_TYPE(C, function) \
    has_ ## function<C>

DEFINE_MEMBER_CHECKER(foo)
DEFINE_FUNCTION_CHECKER(test)
DEFINE_FUNCTION_CHECKER(check)

template <typename Type>
struct TestClass
{
    template <typename U = Type, typename std::enable_if<HAS_FUNCTION(U, test)>::type* = nullptr>
    void test() { std::cout << "this have test()" << std::endl; }
};

int main()
{
    std::cout << std::boolalpha;
    std::cout << HAS_MEMBER(X, foo) << std::endl;
    std::cout << HAS_MEMBER(Y, foo) << std::endl;
    std::cout << HAS_FUNCTION(X, test) << std::endl;
    std::cout << HAS_FUNCTION(Y, test) << std::endl;
    std::cout << HAS_FUNCTION(X, check) << std::endl;
    std::cout << HAS_FUNCTION(Y, check) << std::endl;
    
    TestClass<X> x;
    x.test();
    TestClass<Y> y;
//    y.test(); // error
}
