import flask
import requests
import secrets

APP = flask.Flask(__name__)
# Lol don't do this
APP.secret_key = secrets.token_bytes(16)

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
    member_teams = list(filter(lambda x: x['name'] is not None, r.json()))
    r = requests.get('http://localhost:8080/api/team/manage/admin/get', params= {'uid': 'User::"{0}"'.format(flask.session['username'])})
    admin_teams = list(filter(lambda x: x['name'] is not None, r.json()))
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
    return render_page('list/index.html', id=l['uid'], name=l['name'], items=l['tasks'], readers=l['readers'], editors=l['editors'])

@APP.route('/list/add_item', methods=['POST'])
def add_item():
    user = 'User::"{0}"'.format(flask.session['username'])
    list_uid = flask.request.form['name']
    task_name = flask.request.form['item']
    data = {
        'uid': user,
        'list': list_uid,
        'name': task_name,
    }
    r = requests.post('http://localhost:8080/api/task/create', json=data)
    if r.status_code == 200:
        body = r.json()
        if is_error(body):
            if is_authz_denied(body):
                return flask.redirect(f'/auth_denied')
            else:
                print('Error: %s' % body['error'])
    return flask.redirect(f'/lists')

@APP.route('/list/delete_task', methods=['POST'])
def delete_item():
    user = 'User::"{0}"'.format(flask.session['username'])
    list_uid = flask.request.form['name']
    task_name = flask.request.form['task_id']
    data = {
        'uid': user,
        'list': list_uid,
        'task': task_name,
    }
    r = requests.delete('http://localhost:8080/api/task/delete', json=data)
    print(r)
    print(data)
    if r.status_code == 200:
        body = r.json()
        if is_error(body):
            if is_authz_denied(body):
                return flask.redirect(f'/auth_denied')
            else:
                print('Error: %s' % body['error'])
    return flask.redirect(f'/lists')

@APP.route('/list/share_with', methods = ['POST'])
def share_with():
    user= 'User::"{0}"'.format(flask.request.form['user'])
    data = {
        'uid': 'User::"{0}"'.format(flask.session['username']),
        'list': flask.request.form['list_id'],
        'share_with': user,
        'role': flask.request.form['share_kind']
    }
    r = requests.post('http://localhost:8080/api/share', json=data)
    if r.status_code == 200:
        body = r.json()
        if is_error(body):
            if is_authz_denied(body):
                return flask.redirect(f'/auth_denied')
            else:
                print('Error: %s' % body['error'])
    return flask.redirect(f'/lists')

@APP.route('/team/create', methods=['GET'])
def get_create_team():
    return render_page('team/create.html')

@APP.route('/team/create', methods=['POST'])
def post_create_team():
    try:
        id = flask.request.form['name']
        owner = flask.session['username']
        data = {'owner': owner, 'id': id}
        requests.post('http://localhost:8080/api/team/create', json=data)
        return flask.redirect(f'/teams')
    except KeyError:
        return 'Bad args!', 403

@APP.route('/auth_denied')
def auth_denied():
    return 'You are not authorized to perform this action', 403

@APP.route('/team/read')
def get_team():
    uid = flask.request.args['uid']
    r = requests.get('http://localhost:8080/api/team/get', params= {'uid': uid})
    l = r.json()
    return render_page('team/index.html', id=l['uid'], name=l['name'],)

@APP.route('/team/remove_admin', methods=['POST'])
def remove_admin():
    data = {
        'team': flask.request.form['name'],
        'user': flask.session['username'],
        'candidate': flask.request.form['candidate']
    }
    r = requests.delete('http://localhost:8080/api/team/admin/remove', json=data)
    if r.status_code == 200:
        body = r.json()
        if is_error(body):
            if is_authz_denied(body):
                return flask.redirect(f'/auth_denied')
            else:
                print('Error: %s' % body['error'])
    return flask.redirect(f'/teams')

@APP.route('/team/add_admin', methods=['POST'])
def add_admin():
    data = {
        'team': flask.request.form['name'],
        'user': flask.session['username'],
        'candidate': flask.request.form['candidate']
    }
    requests.post('http://localhost:8080/api/team/admin/add', json=data)
    return flask.redirect(f'/teams')

@APP.route('/team/remove_member', methods=['POST'])
def remove_member():
    data = {
        'team': flask.request.form['name'],
        'user': flask.session['username'],
        'candidate': flask.request.form['candidate']
    }
    r = requests.delete('http://localhost:8080/api/team/member/remove', json=data)
    if r.status_code == 200:
        body = r.json()
        if is_error(body):
            if is_authz_denied(body):
                return flask.redirect(f'/auth_denied')
            else:
                print('Error: %s' % body['error'])
    return flask.redirect(f'/teams')

@APP.route('/team/add_member', methods=['POST'])
def add_member():
    data = {
        'team': flask.request.form['name'],
        'user': flask.session['username'],
        'candidate': flask.request.form['candidate']
    }
    requests.post('http://localhost:8080/api/team/member/add', json=data)
    return flask.redirect(f'/teams')

def is_error(body):
    return type(body) is dict and 'error' in body

def is_authz_denied(body):
    return 'error' in body and body['error'] == 'Authorization Denied'

if __name__ == '__main__':
    APP.debug = True
    APP.run()
