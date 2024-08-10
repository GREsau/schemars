FROM ruby:2.7.4

EXPOSE 4000

RUN gem update --system 3.4.22 && echo 'url: "http://localhost:4000"' > /_config.localhost.yml

COPY Gemfile .

RUN bundle install

ENV JEKYLL_ENV=docker

ENTRYPOINT [ "bundle", "exec", "jekyll"]
CMD [ "serve", "--force_polling", "--host", "0.0.0.0", "--config", "./_config.yml,/_config.localhost.yml" ]
