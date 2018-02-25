#include <type_traits>

template<typename T, typename = void>
struct has_id : std::false_type { };

template<typename T>
struct has_id<T, decltype(std::declval<T>().id, void())> : std::true_type { };

#include <iostream>

using namespace std;

struct X { int id; };
struct Y { int foo; };

int main()
{
    cout << boolalpha;
    cout << has_id<X>::value << endl;
    cout << has_id<Y>::value << endl;
}
