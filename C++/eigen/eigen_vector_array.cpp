#include <iostream>
#include <Eigen/Eigen>

using namespace std;
using namespace Eigen;

int main(int argc, char *argv[]) 
{
    Vector3f v1 {1., 2., 3.};
    Vector3f v2 {4., 5., 6.};
    
    Vector3f v;
    v = v1.array() * v2.array();
//    Vector3f v = v1 * v2; // error
    
    cout << v1 << endl << endl;
    cout << v2 << endl << endl;
    cout << v << endl;
}