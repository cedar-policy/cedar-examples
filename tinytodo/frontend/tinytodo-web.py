import flask
import requests
import json

APP = flask.Flask(__name__)
# Lol don't do this
APP.secret_key = b'1234'

def render_page(page, **kwargs):
    if 'username' in flask.session:
        print('is logged in')
        return flask.render_template(page, user=flask.session['username'], **kwargs)
    else:
        return flask.render_template(page, **kwargs)

@APP.route("/")
def index():
    return render_page('index.html')

@APP.route('/login', methods=['GET', 'POST'])
def login():
    if flask.request.method == 'POST':
        username = flask.request.form['username']
        flask.session['username'] = username
        return flask.redirect(flask.url_for('index'))
    else:
        return render_page('login.html')
    
@APP.route("/logout")
def logout():
    flask.session.pop('username', None)
    return flask.redirect(flask.url_for('index'))

@APP.route('/lists/')
def get_lists():
    r = requests.get('http://localhost:8080/api/lists/get', params= {'uid': 'User::"{0}"'.format(flask.session['username'])})
    return render_page('lists.html', lists=r.json())

@APP.route('/teams')
def get_teams():
    r = requests.get('http://localhost:8080/api/team/manage/member/get', params= {'uid': 'User::"{0}"'.format(flask.session['username'])})
    member_teams = r.json()
    r = requests.get('http://localhost:8080/api/team/manage/admin/get', params= {'uid': 'User::"{0}"'.format(flask.session['username'])})
    admin_teams = r.json()
    return render_page('teams.html', member_teams=member_teams, admin_teams=admin_teams)

@APP.route('/list/create', methods=['GET'])
def get_create_list():
    return render_page('list/create.html')

@APP.route('/list/create', methods=['POST'])
def post_create_list():
    try:
        name = flask.request.form['name']
        owner = 'User::"{0}"'.format(flask.session['username'])
        data = {'uid': owner, 'name': name}
        requests.post('http://localhost:8080/api/list/create', json=data)
        return flask.redirect(f'/lists')
    except KeyError:
        return 'Bad args!', 403
    
@APP.route('/list/read')
def get_list():
    list = flask.request.args['name']
    r = requests.get('http://localhost:8080/api/list/get', params= {'uid': 'User::"{0}"'.format(flask.session['username']), 'list': list})
    l = r.json()
    print(l)
    return render_page('list/index.html', name=l['name'], items=l['tasks'], readers=l['readers'], editors=l['editors'])
    

if __name__ == '__main__':
    APP.debug = True
    APP.run()