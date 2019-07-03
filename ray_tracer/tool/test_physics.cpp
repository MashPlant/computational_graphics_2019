#include "physics.h"
#include <GL/gl.h>
#include <GL/glu.h>
#include <GL/glut.h>
#include <sstream>
#include <string>

int ds_type = 0;

int N = 1500, col_cnt = 0;

float min_pos = -150, max_pos = 150;
float min_v = -150, max_v = 150;
float min_r = 1.25f, max_r = 7.5f;
const float spf = 1.0 / 60.0;

int screen_width = 1200, screen_height = 900;
float rotate_x, rotate_y;
PhysicsEmulator pe;

void on_size_change(GLsizei w, GLsizei h) {
  screen_width = w, screen_height = h;
  GLfloat range = max_pos - min_pos;
  if (!h)
    h = 1;
  glViewport(0, 0, w, h);
  glMatrixMode(GL_PROJECTION);
  glLoadIdentity();
  if (w <= h)
    glOrtho(-range, range, -range * h / w, range * h / w, -range, range);
  else
    glOrtho(-range * w / h, range * w / h, -range, range, -range, range);
  glMatrixMode(GL_MODELVIEW);
  glLoadIdentity();
}

void init() {
  pe.bound[0][0] = pe.bound[1][0] = pe.bound[2][0] = min_pos;
  pe.bound[0][1] = pe.bound[1][1] = pe.bound[2][1] = max_pos;
  pe.g = Vec3{0, 0, -10};
  pe.G = 10000;
  pe.step = 1e-4;
  for (int i = 0; i < 125; ++i) {
    int x = i / 25, y = i % 25 / 5, z = i % 25 % 5;
    pe.ss.push_back(MovingSphere{Sphere{Vec3{x * 20, y * 20, z * 20}, 8},
                                 Vec3{rand() % 10, rand() % 10, rand() % 10},
                                 1});
  }
  //   pe.ss.push_back(MovingSphere{Sphere{Vec3{20}, 8}, Vec3{0, 10}, 1});
  //   pe.ss.push_back(MovingSphere{Sphere{Vec3{-20}, 8}, Vec3{0, -10}, 1});
  glClearColor(0.0f, 0.0f, 1.0f, 1.0f);
}

void tick() {
  for (int _ = 0; _ < 1000; ++_) {
    pe.next();
  }
  glutPostRedisplay();
}

static void print_text(const std::string &msg, int x, int y) {
  glMatrixMode(GL_PROJECTION);
  glPushMatrix();
  glLoadIdentity();

  gluOrtho2D(0, screen_width, 0, screen_height);
  glScalef(1, -1, 1);
  glTranslatef(0, -screen_height, 0);

  glMatrixMode(GL_MODELVIEW);
  glPushMatrix();
  glLoadIdentity();

  glColor3f(1.0f, 1.0f, 0.0f);

  glRasterPos2f(x, y);

  for (int i = 0; i < msg.size(); i++)
    glutBitmapCharacter(GLUT_BITMAP_HELVETICA_18, msg[i]);

  glColor3f(1.0f, 0.0f, 0.0f);

  glPopMatrix();
  glMatrixMode(GL_PROJECTION);
  glPopMatrix();
  glMatrixMode(GL_MODELVIEW);
}

void render_info() {
  static int last_time = 0, frame_cnt = 0;
  static float fps = 0, sum_col_cnt = 0, ave_col_cnt = 0;
  ++frame_cnt, sum_col_cnt += col_cnt;
  int time = glutGet(GLUT_ELAPSED_TIME);
  if (time - last_time > 1000) {
    fps = frame_cnt * 1000.0f / (time - last_time);
    ave_col_cnt = sum_col_cnt / frame_cnt;
    last_time = time, frame_cnt = 0, sum_col_cnt = 0;
  }
  print_text("fps: " + std::to_string(fps), 20, 75);
}

void on_render() {
  tick();

  glClear(GL_COLOR_BUFFER_BIT);

  render_info();

  glRotatef(rotate_x, 1.0f, 0.0f, 0.0f), rotate_x = 0;
  glRotatef(rotate_y, 0.0f, 1.0f, 0.0f), rotate_y = 0;
  for (let &p : pe.ss) {
    glPushMatrix();
    glTranslatef(p.c.x, p.c.y, p.c.z);
    // if (p.color)
    glColor3f(1.0f, 0.0f, 0.0f);
    // else
    //   glColor3f(1.0f, 1.0f, 1.0f);
    glutSolidCube(p.r);
    glPopMatrix();
  }
  //   for (auto &p : point_entity) {
  //     glPushMatrix();
  //     glTranslatef(p.pos.x, p.pos.y, p.pos.z);
  //     if (p.color)
  //       glColor3f(1.0f, 0.0f, 0.0f);
  //     else
  //       glColor3f(1.0f, 1.0f, 1.0f);
  //     glutSolidCube(p.radius);
  //     glPopMatrix();
  //   }
  // box
  glColor3f(1.0f, 1.0f, 1.0f);

  glEnable(GL_LINE_STIPPLE);
  glLineStipple(2, 0x0F0F);

  glBegin(GL_LINE_STRIP);

  glVertex3f(min_pos, max_pos, min_pos);
  glVertex3f(max_pos, max_pos, min_pos);
  glVertex3f(max_pos, max_pos, max_pos);
  glVertex3f(min_pos, max_pos, max_pos);
  glVertex3f(min_pos, max_pos, min_pos);

  glEnd();

  glBegin(GL_LINE_STRIP);

  glVertex3f(min_pos, min_pos, min_pos);
  glVertex3f(max_pos, min_pos, min_pos);
  glVertex3f(max_pos, min_pos, max_pos);
  glVertex3f(min_pos, min_pos, max_pos);
  glVertex3f(min_pos, min_pos, min_pos);

  glEnd();

  glBegin(GL_LINES);

  glVertex3f(min_pos, max_pos, max_pos);
  glVertex3f(min_pos, min_pos, max_pos);

  glVertex3f(min_pos, max_pos, min_pos);
  glVertex3f(min_pos, min_pos, min_pos);

  glVertex3f(max_pos, max_pos, min_pos);
  glVertex3f(max_pos, min_pos, min_pos);

  glVertex3f(max_pos, max_pos, max_pos);
  glVertex3f(max_pos, min_pos, max_pos);

  glEnd();

  glDisable(GL_LINE_STIPPLE);

  glutSwapBuffers();
}

void on_mouse_drag(int x, int y) {
  static int last_x = -1, last_y;
  if (last_x != -1)
    rotate_x = (last_x - x) * 0.5, rotate_y = (last_y - y) * 0.5;
  last_x = x, last_y = y;
}

void on_key_down(unsigned char ch_key, int, int) {}

int main(int argc, char **argv) {
  glutInit(&argc, argv);
  glutInitDisplayMode(GLUT_DOUBLE | GLUT_RGB);
  glutInitWindowSize(screen_width, screen_height);
  glutCreateWindow("SpatialDS");
  glutDisplayFunc(on_render);
  glutReshapeFunc(on_size_change);
  glutMotionFunc(on_mouse_drag);
  glutKeyboardFunc(on_key_down);
  init();
  glutMainLoop();
  return 0;
}
