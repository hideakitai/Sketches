#include <iostream>

using namespace std;
using namespace Eigen;

int main(int argc, char *argv[]) 
{
    Vector3f one = Vector3f::Ones();
    Vector3f two = {1., 2, 3,};
    cout << one << endl << endl;
    cout << two << endl << endl;
    
    Vector3f mul = one.array() * two.array();
    cout << mul << endl;
    
    Vector3d v {1, 2, 3};
    cout << v << endl;
    cout << v(0) << ", " << v(1) << ", " << v(2) << endl;
}