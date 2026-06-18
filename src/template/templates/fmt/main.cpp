// {{project_name}} - fmt formatting example
#include <fmt/core.h>
#include <fmt/color.h>
#include <fmt/ranges.h>
#include <vector>
#include <string>

int main() {
    fmt::print("Hello from {}!\n", "{{project_name}}");

    // Colored output
    fmt::print(fg(fmt::color::green), "This is green text\n");
    fmt::print(fg(fmt::color::cyan) | fmt::emphasis::bold, "Bold cyan!\n");

    // Formatting numbers
    fmt::print("Pi is approximately {:.4f}\n", 3.14159265);
    fmt::print("Hex: {:#x}, Octal: {:#o}, Binary: {:#b}\n", 255, 255, 255);

    // Formatting containers
    std::vector<int> numbers = {1, 2, 3, 4, 5};
    fmt::print("Numbers: {}\n", fmt::join(numbers, ", "));

    // Named arguments
    fmt::print("{name} is {age} years old\n",
        fmt::arg("name", "Alice"),
        fmt::arg("age", 30));

    return 0;
}
