// {{project_name}} - Boost.Filesystem example
#include <iostream>
#include <fstream>
#include <boost/filesystem.hpp>

namespace fs = boost::filesystem;

int main() {
    fs::path current = fs::current_path();
    std::cout << "Current directory: " << current << std::endl;
    std::cout << "Parent: " << current.parent_path() << std::endl;
    std::cout << std::endl;

    // List files in current directory
    std::cout << "Contents:" << std::endl;
    for (const auto& entry : fs::directory_iterator(current)) {
        auto status = entry.status();
        std::string type_str = fs::is_directory(status) ? "[DIR] " : "      ";
        auto size = fs::is_regular_file(status) ? fs::file_size(entry.path()) : 0;

        std::cout << "  " << type_str << entry.path().filename().string();
        if (size > 0) {
            std::cout << " (" << size << " bytes)";
        }
        std::cout << std::endl;
    }

    // Create a temp directory
    fs::path temp = current / "temp_demo";
    if (!fs::exists(temp)) {
        fs::create_directory(temp);
        std::cout << "\nCreated: " << temp << std::endl;
    }

    // Write and read a file
    fs::path file = temp / "hello.txt";
    {
        std::ofstream out(file.string());
        out << "Hello from {{project_name}}!";
    }
    std::cout << "Wrote: " << file << " (" << fs::file_size(file) << " bytes)" << std::endl;

    // Cleanup
    fs::remove_all(temp);
    std::cout << "Cleaned up temp directory" << std::endl;

    return 0;
}
