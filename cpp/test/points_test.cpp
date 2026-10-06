// Minimal test binary: non-zero exit on first failed assertion.
#include <cassert>
#include <cmath>
#include <string>
#include <vector>

#include "routeopt/points.hpp"

int main() {
  // render/parse round trip
  routeopt::Point p{42, 10.25, -3.5};
  const auto text = routeopt::render_point(p);
  assert(text == "42 10.2 -3.5"); // %.1f truncates to one decimal

  routeopt::Point back{};
  assert(routeopt::parse_point(text, &back));
  assert(back.id == 42);
  assert(std::fabs(back.x - 10.2) < 1e-9);
  assert(std::fabs(back.y + 3.5) < 1e-9);

  // malformed lines rejected
  routeopt::Point junk{};
  assert(!routeopt::parse_point("not a point", &junk));
  assert(!routeopt::parse_point("1 2", &junk));

  // path length: 3-4-5 legs
  std::vector<routeopt::Point> tri{
      {0, 0.0, 0.0}, {1, 3.0, 0.0}, {2, 3.0, 4.0}};
  assert(std::fabs(routeopt::path_length(tri) - 7.0) < 1e-9);

  return 0;
}
