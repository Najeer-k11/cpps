// {{project_name}} - Main source
#include <iostream>

int add(int a, int b) {
    return a + b;
}

int main() {
    std::cout << "{{project_name}}: 2 + 3 = " << add(2, 3) << std::endl;
    return 0;
}
