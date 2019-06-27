/*
  Copyright (c) 2009, Hideyuki Tanaka
  All rights reserved.

  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions are met:
  * Redistributions of source code must retain the above copyright
  notice, this list of conditions and the following disclaimer.
  * Redistributions in binary form must reproduce the above copyright
  notice, this list of conditions and the following disclaimer in the
  documentation and/or other materials provided with the distribution.
  * Neither the name of the <organization> nor the
  names of its contributors may be used to endorse or promote products
  derived from this software without specific prior written permission.

  THIS SOFTWARE IS PROVIDED BY <copyright holder> ''AS IS'' AND ANY
  EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
  WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
  DISCLAIMED. IN NO EVENT SHALL <copyright holder> BE LIABLE FOR ANY
  DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
  (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
  LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
  ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
  SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

#include "cmdline.h"

namespace cmdline
{
  parser::~parser()
  {
    for (std::map<std::string, option_base *>::iterator p = options.begin();
         p != options.end(); p++)
      delete p->second;
  }

  void parser::add(const std::string &name,
                   char short_name /*= 0*/,
                   const std::string &desc /*= ""*/)
  {
    if (options.count(name)) throw cmdline_error("multiple definition: " + name);
    options[name] = new option_without_value(name, short_name, desc);
    ordered.push_back(options[name]);
  }

  void parser::footer(const std::string &f)
  {
    ftr = f;
  }

  void parser::set_program_name(const std::string &name)
  {
    prog_name = name;
  }

  bool parser::exist(const std::string &name) const
  {
    if (options.count(name) == 0) throw cmdline_error("there is no flag: --" + name);
    return options.find(name)->second->has_set();
  }

  const std::vector<std::string> &parser::rest() const
  {
    return others;
  }

  bool parser::parse(const std::string &arg)
  {
    std::vector<std::string> args;

    std::string buf;
    bool in_quote = false;
    for (std::string::size_type i = 0; i < arg.length(); i++)
    {
      if (arg[i] == '\"')
      {
        in_quote = !in_quote;
        continue;
      }

      if (arg[i] == ' ' && !in_quote)
      {
        args.push_back(buf);
        buf = "";
        continue;
      }

      if (arg[i] == '\\')
      {
        i++;
        if (i >= arg.length())
        {
          errors.push_back("unexpected occurrence of '\\' at end of string");
          return false;
        }
      }

      buf += arg[i];
    }

    if (in_quote)
    {
      errors.push_back("quote is not closed");
      return false;
    }

    if (buf.length() > 0)
      args.push_back(buf);

    for (size_t i = 0; i < args.size(); i++)
      std::cout << "\"" << args[i] << "\"" << std::endl;

    return parse(args);
  }

  bool parser::parse(const std::vector<std::string> &args)
  {
    int argc = static_cast<int>(args.size());
    std::vector<const char *> argv(argc);

    for (int i = 0; i < argc; i++)
      argv[i] = args[i].c_str();

    return parse(argc, &argv[0]);
  }

  bool parser::parse(int argc, const char *const argv[])
  {
    errors.clear();
    others.clear();

    if (argc < 1)
    {
      errors.push_back("argument number must be longer than 0");
      return false;
    }
    if (prog_name == "")
      prog_name = argv[0];

    std::map<char, std::string> lookup;
    for (std::map<std::string, option_base *>::iterator p = options.begin();
         p != options.end(); p++)
    {
      if (p->first.length() == 0) continue;
      char initial = p->second->short_name();
      if (initial)
      {
        if (lookup.count(initial) > 0)
        {
          lookup[initial] = "";
          errors.push_back(std::string("short option '") + initial + "' is ambiguous");
          return false;
        }
        else lookup[initial] = p->first;
      }
    }

    for (int i = 1; i < argc; i++)
    {
      if (strncmp(argv[i], "--", 2) == 0)
      {
        const char *p = strchr(argv[i] + 2, '=');
        if (p)
        {
          std::string name(argv[i] + 2, p);
          std::string val(p + 1);
          set_option(name, val);
        }
        else
        {
          std::string name(argv[i] + 2);
          if (options.count(name) == 0)
          {
            errors.push_back("undefined option: --" + name);
            continue;
          }
          if (options[name]->has_value())
          {
            if (i + 1 >= argc)
            {
              errors.push_back("option needs value: --" + name);
              continue;
            }
            else
            {
              i++;
              set_option(name, argv[i]);
            }
          }
          else
          {
            set_option(name);
          }
        }
      }
      else if (strncmp(argv[i], "-", 1) == 0)
      {
        if (!argv[i][1]) continue;
        char last = argv[i][1];
        for (int j = 2; argv[i][j]; j++)
        {
          last = argv[i][j];
          if (lookup.count(argv[i][j - 1]) == 0)
          {
            errors.push_back(std::string("undefined short option: -") + argv[i][j - 1]);
            continue;
          }
          if (lookup[argv[i][j - 1]] == "")
          {
            errors.push_back(std::string("ambiguous short option: -") + argv[i][j - 1]);
            continue;
          }
          set_option(lookup[argv[i][j - 1]]);
        }

        if (lookup.count(last) == 0)
        {
          errors.push_back(std::string("undefined short option: -") + last);
          continue;
        }
        if (lookup[last] == "")
        {
          errors.push_back(std::string("ambiguous short option: -") + last);
          continue;
        }

        if (i + 1 < argc && options[lookup[last]]->has_value())
        {
          set_option(lookup[last], argv[i + 1]);
          i++;
        }
        else
        {
          set_option(lookup[last]);
        }
      }
      else
      {
        others.push_back(argv[i]);
      }
    }

    for (std::map<std::string, option_base *>::iterator p = options.begin();
         p != options.end(); p++)
      if (!p->second->valid())
        errors.push_back("need option: --" + std::string(p->first));

    return errors.size() == 0;
  }

  void parser::parse_check(const std::string &arg)
  {
    if (!options.count("help"))
      add("help", '?', "print this message");
    check(0, parse(arg));
  }

  void parser::parse_check(const std::vector<std::string> &args)
  {
    if (!options.count("help"))
      add("help", '?', "print this message");
    check(args.size(), parse(args));
  }

  void parser::parse_check(int argc, char *argv[])
  {
    if (!options.count("help"))
      add("help", '?', "print this message");
    check(argc, parse(argc, argv));
  }

  std::string parser::error() const
  {
    return errors.size() > 0 ? errors[0] : "";
  }

  std::string parser::error_full() const
  {
    std::ostringstream oss;
    for (size_t i = 0; i < errors.size(); i++)
      oss << errors[i] << std::endl;
    return oss.str();
  }

  std::string parser::usage() const
  {
    std::ostringstream oss;
    oss << "usage: " << prog_name << " ";
    for (size_t i = 0; i < ordered.size(); i++)
    {
      if (ordered[i]->must())
        oss << ordered[i]->short_description() << " ";
    }

    oss << "[options] ... " << ftr << std::endl;
    oss << "options:" << std::endl;

    size_t max_width = 0;
    for (size_t i = 0; i < ordered.size(); i++)
    {
      max_width = std::max(max_width, ordered[i]->name().length());
    }
    for (size_t i = 0; i < ordered.size(); i++)
    {
      if (ordered[i]->short_name())
      {
        oss << "  -" << ordered[i]->short_name() << ", ";
      }
      else
      {
        oss << "      ";
      }

      oss << "--" << ordered[i]->name();
      for (size_t j = ordered[i]->name().length(); j < max_width + 4; j++)
        oss << ' ';
      oss << ordered[i]->description() << std::endl;
    }
    return oss.str();
  }

  void parser::check(int argc, bool ok)
  {
    if ((argc == 1 && !ok) || exist("help"))
    {
      std::cerr << usage();
      exit(0);
    }

    if (!ok)
    {
      std::cerr << error() << std::endl << usage();
      exit(1);
    }
  }

  void parser::set_option(const std::string &name)
  {
    if (options.count(name) == 0)
    {
      errors.push_back("undefined option: --" + name);
      return;
    }
    if (!options[name]->set())
    {
      errors.push_back("option needs value: --" + name);
      return;
    }
  }

  void parser::set_option(const std::string &name, const std::string &value)
  {
    if (options.count(name) == 0)
    {
      errors.push_back("undefined option: --" + name);
      return;
    }
    if (!options[name]->set(value))
    {
      errors.push_back("option value is invalid: --" + name + "=" + value);
      return;
    }
  }
}