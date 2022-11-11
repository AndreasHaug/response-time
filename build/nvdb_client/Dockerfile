FROM python:3.10.8


# RUN groupadd app
# RUN useradd -g app app
# RUN usermod -aG docker app
# USER app

# RUN mkdir DATA
# RUN ls

WORKDIR /client
COPY ./requirements.txt .
COPY ./get_data_selection.py .

CMD python3 get_data_selection.py
