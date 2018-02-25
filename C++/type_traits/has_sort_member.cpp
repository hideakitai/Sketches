#include <type_traits>
#include <utility>
#include <iostream>
#include <iterator>
#include <vector>
#include <list>
#include <algorithm>

struct has_sort_member_impl
{
    template <class T>
    static auto check(T*) -> decltype(std::declval<T>().sort(), std::true_type());

    template <class T>
    static auto check(...) -> std::false_type;
};

template <class T>
struct has_sort_member : decltype(has_sort_member_impl::check<T>(nullptr)) {};


template <class Sortable, typename std::enable_if<has_sort_member<Sortable>::value>::type* = nullptr>
void sort(Sortable& x)
{
    x.sort();
}

template <class Range, typename std::enable_if<!has_sort_member<Range>::value>::type* = nullptr>
void sort(Range& r)
{
    std::sort(std::begin(r), std::end(r));
}


int main()
{
    std::vector<int> v = {3, 1, 4};
    std::list<int> ls = {3, 1, 4};

    sort(v);
    sort(ls);

    for (int x : v) std::cout << x << " ";
    std::cout << std::endl;

    for (int x : ls) std::cout << x << " ";
}

