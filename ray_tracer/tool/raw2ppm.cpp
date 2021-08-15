#include <cassert>
#include "tracer_util.hpp"

size_t file_size(FILE *f) {
  fseek(f, 0, SEEK_END);
  size_t size = ftell(f);
  fseek(f, 0, SEEK_SET);
  return size;
}

struct Raw {
  u32 w, h;
  Vec3 raw[0];
};

int main(int argc, char **args) {
  assert(argc > 1);
  size_t size;
  {
    FILE *f = fopen(args[1], "r");
    assert(f);
    size = file_size(f);
    assert(size > 8 && (size - 8) % sizeof(Vec3) == 0);
    fclose(f);
  }
  size_t n = (size - 8) / sizeof(Vec3);
  Raw *sum = (Raw *)calloc(1, size), *tmp = (Raw *)malloc(size);
  int n_pic = 0;
  const char *out_path = "image.ppm";
  for (int i = 1; i < argc; ++i) {
    if (args[i][0] == '-') {
      assert(args[i][1] == 'o');
      assert(args[i][2] == '=');
      out_path = &args[i][3];
      continue;
    }
    ++n_pic;
    FILE *f = fopen(args[i], "r");
    assert(f);
    assert(file_size(f) == size);
    assert(fread(tmp, 1, size, f) == size);
    if (sum->w == 0) {
      sum->w = tmp->w, sum->h = tmp->h;
      assert(sum->w && sum->h);
    } else {
      assert(sum->w == tmp->w && sum->h == tmp->h);
    }
    for (size_t j = 0; j < n; ++j) {
      sum->raw[j] += tmp->raw[j];
    }
    fclose(f);
  }
  assert(n_pic);
  for (size_t i = 0; i < n; ++i) {
    sum->raw[i] /= n_pic;
  }
  output_ppm(sum->raw, sum->w, sum->h, out_path);
  free(sum), free(tmp);
}