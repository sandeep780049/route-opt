// Standalone utility: reads "id x y" lines from stdin, echoes the valid
// points back and prints the open-path length of the sequence.
#include <iostream>
#include <string>

#include "routeopt/points.hpp"

int main() {
  std::vector<routeopt::Point> pts;
  std::string line;
  while (std::getline(std::cin, line)) {
    if (line.empty()) {
      continue;
    }
    routeopt::Point p;
    if (routeopt::parse_point(line, &p)) {
      pts.push_back(p);
    } else {
      std::cerr << "skipping malformed line: " << line << '\n';
    }
  }

  for (const auto &p : pts) {
    std::cout << routeopt::render_point(p) << '\n';
  }
  std::cout << "path_length " << routeopt::path_length(pts) << '\n';
  return 0;
}
