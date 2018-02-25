#include <iostream>
#include <Eigen/Core>
#include <Eigen/Geometry>

// http://blog.livedoor.jp/tek_nishi/archives/8623876.html

int main() 
{
    using namespace Eigen;

    // 変数を定義。値は未定義
    Quaternionf q1;

    // コンストラクタで値(w, x, y, z)を渡して初期化
    Quaternionf q2(1.0f, 0.0f, 0.25f, 0.5f);
    std::cout << q2.w() << "," << q2.x() << "," << q2.y() << "," << q2.z() << std::endl;

    // コンストラクタで角度とベクトルを渡して初期化
    Quaternionf q3(AngleAxisf(0.1f, Vector3f::UnitY()));

    // 2つのベクトルからクオータニオンを求める
    Quaternionf q4 = Quaternionf::FromTwoVectors(Vector3f::UnitX(), Vector3f::UnitZ());

    // 単位クオータニオン(wが1で他が0)
    q4 = Quaternionf::Identity();
    std::cout << q4.w() << "," << q4.x() << "," << q4.y() << "," << q4.z() << std::endl;

    // 乗算
    Quaternionf q_mul = q2 * q3;

    // 逆クオータニオン
    Quaternionf q_inv = q4.inverse();

    // 共役クオータニオンを求める
    Quaternionf q_conj = q4.conjugate();

    // 内積
    float dot = q3.dot(q4);

    // 回転ベクトルの長さ
    float norm = q3.norm();

    // 正規化
    q3.normalize();
    Quaternionf q_normalized = q4.normalized();

    // 球面線形補間
    // q3→q4を t[0, 1.0] で補間する
    float t = 0.5f;
    Quaternionf q_slerp = q3.slerp(t, q4);
}