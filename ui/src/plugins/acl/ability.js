import { Ability } from '@casl/ability'
import store from '../../store'
import { initialAbility } from './config'

const existingAbility = store.state.userAbility

export default new Ability(existingAbility || initialAbility)
